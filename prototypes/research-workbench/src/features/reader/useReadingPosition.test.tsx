import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import type {
  IpcResult,
  ReadingPositionDto,
} from "../../shared/contracts/generated/contracts";
import type { ReaderApi } from "../../shared/contracts/ports";
import { useReadingPosition } from "./useReadingPosition";

afterEach(cleanup);
const resultId = "00000000-0000-4000-8000-000000000099";
const position = (
  documentId: string,
  revision: number,
  pageIndex = 1,
): ReadingPositionDto => ({
  documentId,
  pageIndex,
  zoom: 1,
  revision,
  updatedAt: "2026-10-02T00:00:00Z",
});
const ok = <T,>(data: T): IpcResult<T> => ({
  contractVersion: 1,
  requestId: resultId,
  ok: true,
  data,
});
const fail: IpcResult<never> = {
  contractVersion: 1,
  requestId: resultId,
  ok: false,
  error: {
    code: "StorageUnavailable",
    message: "Could not confirm.",
    retryable: true,
  },
};
const conflict: IpcResult<never> = {
  contractVersion: 1,
  requestId: resultId,
  ok: false,
  error: {
    code: "Conflict",
    message: "The position changed in another operation.",
    retryable: false,
  },
};

it("conflict_stops_prequeued_saves_instead_of_rebasing_them", async () => {
  const documentId = "00000000-0000-4000-8000-000000000111";
  const save = vi
    .fn()
    .mockResolvedValueOnce(conflict)
    .mockResolvedValueOnce(ok(position(documentId, 2, 3)));
  const get = vi.fn().mockResolvedValue(ok(position(documentId, 1, 9)));
  const api = {
    saveReadingPosition: save,
    getReadingPosition: get,
  } as unknown as ReaderApi;
  const hook = renderHook(() =>
    useReadingPosition(api, position(documentId, 0, 1)),
  );
  let first!: Promise<ReadingPositionDto>;
  let second!: Promise<ReadingPositionDto>;
  act(() => {
    first = hook.result.current.save(2, 1);
    second = hook.result.current.save(3, 1);
  });
  await act(async () => {
    await Promise.allSettled([first, second]);
  });
  expect(save).toHaveBeenCalledTimes(1);
  expect(get).toHaveBeenCalledTimes(1);
  expect(hook.result.current.status).toBe("error");
  expect(hook.result.current.recovery).toMatchObject({
    kind: "conflict",
    durable: position(documentId, 1, 9),
  });
  const rejected = save.mock.calls[0][0];
  await act(async () => {
    await hook.result.current.resolveConflict("save-local");
  });
  expect(save).toHaveBeenCalledTimes(2);
  expect(save.mock.calls[1][0]).toMatchObject({
    documentId,
    expectedRevision: 1,
    pageIndex: 3,
  });
  expect(save.mock.calls[1][0].requestId).not.toBe(rejected.requestId);
});

it("retries_uncertain_save_with_same_receipt_and_requires_explicit_local_resolution", async () => {
  const documentId = "00000000-0000-4000-8000-000000000101";
  const save = vi
    .fn()
    .mockResolvedValueOnce(fail)
    .mockResolvedValueOnce(ok(position(documentId, 1, 2)));
  const get = vi.fn().mockResolvedValue(ok(position(documentId, 1, 2)));
  const api = {
    saveReadingPosition: save,
    getReadingPosition: get,
  } as unknown as ReaderApi;
  const first = renderHook(() =>
    useReadingPosition(api, position(documentId, 0)),
  );
  await act(async () => {
    await expect(first.result.current.save(2, 1)).rejects.toThrow(
      "Could not confirm.",
    );
  });
  await act(async () => {
    await expect(first.result.current.save(3, 1)).rejects.toThrow();
  });
  expect(save).toHaveBeenCalledTimes(1);
  expect(first.result.current.recovery?.kind).toBe("uncertain");
  await act(async () => {
    await first.result.current.retryUncertain();
  });
  expect(save).toHaveBeenNthCalledWith(
    1,
    expect.objectContaining({ documentId, expectedRevision: 0, pageIndex: 2 }),
  );
  expect(save).toHaveBeenNthCalledWith(
    2,
    expect.objectContaining({
      documentId,
      expectedRevision: 0,
      pageIndex: 2,
      requestId: save.mock.calls[0][0].requestId,
    }),
  );
  expect(get).not.toHaveBeenCalled();
  expect(save).toHaveBeenNthCalledWith(
    2,
    expect.objectContaining({ documentId, expectedRevision: 0, pageIndex: 2 }),
  );
  expect(first.result.current.recovery).toMatchObject({
    kind: "conflict",
    durable: position(documentId, 1, 2),
  });
  save.mockResolvedValueOnce(ok(position(documentId, 2, 3)));
  await act(async () => {
    await first.result.current.resolveConflict("save-local");
  });
  expect(save).toHaveBeenNthCalledWith(
    3,
    expect.objectContaining({ documentId, expectedRevision: 1, pageIndex: 3 }),
  );
  await waitFor(() => expect(first.result.current.status).toBe("saved"));
});

it("a_second_conflict_during_resolution_stays_explicit_and_can_keep_new_durable_state", async () => {
  const documentId = "00000000-0000-4000-8000-000000000122";
  const save = vi
    .fn()
    .mockResolvedValueOnce(conflict)
    .mockResolvedValueOnce(conflict);
  const durableOne = position(documentId, 1, 9);
  const durableTwo = position(documentId, 2, 10);
  const get = vi
    .fn()
    .mockResolvedValueOnce(ok(durableOne))
    .mockResolvedValueOnce(ok(durableTwo));
  const api = {
    saveReadingPosition: save,
    getReadingPosition: get,
  } as unknown as ReaderApi;
  const hook = renderHook(() =>
    useReadingPosition(api, position(documentId, 0, 1)),
  );
  await act(async () => {
    await expect(hook.result.current.save(2, 1)).rejects.toThrow();
  });
  await act(async () => {
    await expect(
      hook.result.current.resolveConflict("save-local"),
    ).rejects.toThrow();
  });
  expect(save).toHaveBeenCalledTimes(2);
  expect(hook.result.current.recovery).toMatchObject({
    kind: "conflict",
    durable: durableTwo,
  });
  let kept!: ReadingPositionDto;
  await act(async () => {
    kept = await hook.result.current.resolveConflict("keep-saved");
  });
  expect(kept).toEqual(durableTwo);
  expect(hook.result.current.recovery).toBeNull();
  expect(save).toHaveBeenCalledTimes(2);
});

it("save_local_resolution_clears_only_its_conflict_and_drains_a_newer_user_save", async () => {
  const documentId = "00000000-0000-4000-8000-000000000131";
  let finishResolution!: (value: IpcResult<ReadingPositionDto>) => void;
  const save = vi
    .fn()
    .mockResolvedValueOnce(conflict)
    .mockImplementationOnce(
      () =>
        new Promise<IpcResult<ReadingPositionDto>>(
          (resolve) => (finishResolution = resolve),
        ),
    )
    .mockResolvedValueOnce(ok(position(documentId, 3, 4)));
  const durable = position(documentId, 1, 9);
  const api = {
    saveReadingPosition: save,
    getReadingPosition: vi.fn().mockResolvedValue(ok(durable)),
  } as unknown as ReaderApi;
  const hook = renderHook(() =>
    useReadingPosition(api, position(documentId, 0)),
  );
  await act(async () => {
    await expect(hook.result.current.save(2, 1)).rejects.toThrow();
  });
  let resolving!: Promise<ReadingPositionDto>;
  act(() => {
    resolving = hook.result.current.resolveConflict("save-local");
  });
  await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  let later!: Promise<ReadingPositionDto>;
  act(() => {
    later = hook.result.current.save(4, 1);
  });
  await act(async () => {
    finishResolution(ok(position(documentId, 2, 2)));
    await resolving;
  });
  await act(async () => {
    await later;
  });
  expect(save).toHaveBeenCalledTimes(3);
  expect(save.mock.calls[2][0]).toMatchObject({
    documentId,
    expectedRevision: 2,
    pageIndex: 4,
  });
  expect(hook.result.current.recovery).toBeNull();
  expect(hook.result.current.status).toBe("saved");
});

it("uncertain_replay_does_not_mark_newer_visible_position_saved", async () => {
  const documentId = "00000000-0000-4000-8000-000000000132";
  const save = vi
    .fn()
    .mockResolvedValueOnce(fail)
    .mockResolvedValueOnce(ok(position(documentId, 1, 2)));
  const api = {
    saveReadingPosition: save,
    getReadingPosition: vi.fn().mockResolvedValue(ok(position(documentId, 1, 2))),
  } as unknown as ReaderApi;
  const hook = renderHook(() =>
    useReadingPosition(api, position(documentId, 0)),
  );
  await act(async () => {
    await expect(hook.result.current.save(2, 1)).rejects.toThrow();
  });
  await act(async () => {
    await expect(hook.result.current.save(3, 1)).rejects.toThrow();
  });
  await act(async () => {
    await hook.result.current.retryUncertain();
  });
  expect(hook.result.current.status).not.toBe("saved");
  expect(hook.result.current.recovery).toMatchObject({
    kind: "conflict",
    intent: { pageIndex: 2 },
    durable: position(documentId, 1, 2),
  });
});

it("late_conflict_from_unmounted_view_reaches_reopened_same_document_hook", async () => {
  const documentId = "00000000-0000-4000-8000-000000000133";
  let finishSave!: (value: IpcResult<ReadingPositionDto>) => void;
  const durable = position(documentId, 1, 9);
  const api = {
    saveReadingPosition: vi.fn(
      () =>
        new Promise<IpcResult<ReadingPositionDto>>(
          (resolve) => (finishSave = resolve),
        ),
    ),
    getReadingPosition: vi.fn().mockResolvedValue(ok(durable)),
  } as unknown as ReaderApi;
  const oldView = renderHook(() =>
    useReadingPosition(api, position(documentId, 0)),
  );
  let pending!: Promise<ReadingPositionDto>;
  act(() => {
    pending = oldView.result.current.save(2, 1);
  });
  oldView.unmount();
  const reopened = renderHook(() =>
    useReadingPosition(api, position(documentId, 1, 9)),
  );
  await waitFor(() => expect(reopened.result.current.recovery).toBeNull());
  await act(async () => {
    finishSave(conflict);
    await expect(pending).rejects.toThrow();
  });
  expect(reopened.result.current.recovery).toMatchObject({
    kind: "conflict",
    durable,
  });
});

it("late_uncertain_result_from_unmounted_view_reaches_reopened_same_document_hook", async () => {
  const documentId = "00000000-0000-4000-8000-000000000134";
  let finishSave!: (value: IpcResult<ReadingPositionDto>) => void;
  const api = {
    saveReadingPosition: vi.fn(
      () =>
        new Promise<IpcResult<ReadingPositionDto>>(
          (resolve) => (finishSave = resolve),
        ),
    ),
    getReadingPosition: vi.fn(),
  } as unknown as ReaderApi;
  const oldView = renderHook(() =>
    useReadingPosition(api, position(documentId, 0)),
  );
  let pending!: Promise<ReadingPositionDto>;
  act(() => {
    pending = oldView.result.current.save(2, 1);
  });
  oldView.unmount();
  const reopened = renderHook(() =>
    useReadingPosition(api, position(documentId, 0)),
  );
  await waitFor(() => expect(reopened.result.current.recovery).toBeNull());
  await act(async () => {
    finishSave(fail);
    await expect(pending).rejects.toThrow("Could not confirm.");
  });
  expect(reopened.result.current.recovery).toMatchObject({
    kind: "uncertain",
    intent: { documentId, pageIndex: 2 },
  });
});

it("late_save_success_for_previous_document_does_not_change_current_view_status", async () => {
  const documentA = "00000000-0000-4000-8000-000000000102";
  const documentB = "00000000-0000-4000-8000-000000000103";
  let release!: (value: IpcResult<ReadingPositionDto>) => void;
  const save = vi.fn(
    () =>
      new Promise<IpcResult<ReadingPositionDto>>((resolve) => {
        release = resolve;
      }),
  );
  const api = {
    saveReadingPosition: save,
    getReadingPosition: vi.fn(),
  } as unknown as ReaderApi;
  const hook = renderHook(({ initial }) => useReadingPosition(api, initial), {
    initialProps: { initial: position(documentA, 0) },
  });
  let pending!: Promise<ReadingPositionDto>;
  act(() => {
    pending = hook.result.current.save(2, 1);
  });
  hook.rerender({ initial: position(documentB, 0) });
  await waitFor(() => expect(hook.result.current.status).toBe("idle"));
  await act(async () => {
    release(ok(position(documentA, 1, 2)));
    await pending;
  });
  expect(hook.result.current.status).toBe("idle");
  expect(hook.result.current.error).toBeNull();
});

it("late_save_error_for_previous_document_does_not_leak_into_reopened_view", async () => {
  const documentA = "00000000-0000-4000-8000-000000000104";
  const documentB = "00000000-0000-4000-8000-000000000105";
  let reject!: (reason: unknown) => void;
  const save = vi.fn(
    () =>
      new Promise<IpcResult<ReadingPositionDto>>((_resolve, no) => {
        reject = no;
      }),
  );
  const api = {
    saveReadingPosition: save,
    getReadingPosition: vi.fn(),
  } as unknown as ReaderApi;
  const hook = renderHook(({ initial }) => useReadingPosition(api, initial), {
    initialProps: { initial: position(documentA, 0) },
  });
  let pending!: Promise<ReadingPositionDto>;
  act(() => {
    pending = hook.result.current.save(2, 1);
  });
  hook.rerender({ initial: position(documentB, 0) });
  await waitFor(() => expect(hook.result.current.status).toBe("idle"));
  await act(async () => {
    reject(new Error("error for A"));
    await expect(pending).rejects.toThrow("error for A");
  });
  expect(hook.result.current.status).toBe("idle");
  expect(hook.result.current.error).toBeNull();
});
