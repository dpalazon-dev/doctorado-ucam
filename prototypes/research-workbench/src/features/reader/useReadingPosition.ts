import { useCallback, useEffect, useRef, useState } from "react";
import type { ReaderApi } from "../../shared/contracts/ports";
import type { ReadingPositionDto } from "../../shared/contracts/generated/contracts";

type SaveIntent = {
  requestId: string;
  documentId: string;
  expectedRevision: number;
  pageIndex: number;
  zoom: number;
};
type LocalPosition = { pageIndex: number; zoom: number };
export type PositionRecovery =
  | { kind: "uncertain"; intent: SaveIntent; message: string }
  | {
      kind: "conflict";
      intent: SaveIntent;
      durable: ReadingPositionDto | null;
      message: string;
    };
type QueueEntry = {
  tail: Promise<void>;
  revision: number;
  uncertain: SaveIntent | null;
  recovery: PositionRecovery | null;
  local: LocalPosition | null;
  listeners: Set<() => void>;
};
const queues = new Map<string, QueueEntry>();

function errorMessage(reason: unknown): string {
  return reason instanceof Error
    ? reason.message
    : "Could not save the position.";
}

function entryFor(documentId: string, revision: number): QueueEntry {
  const existing = queues.get(documentId);
  if (existing) {
    existing.revision = Math.max(existing.revision, revision);
    return existing;
  }
  const entry: QueueEntry = {
    tail: Promise.resolve(),
    revision,
    uncertain: null,
    recovery: null,
    local: null,
    listeners: new Set(),
  };
  queues.set(documentId, entry);
  return entry;
}

async function readDurable(
  api: ReaderApi,
  documentId: string,
): Promise<ReadingPositionDto | null> {
  try {
    const result = await api.getReadingPosition({
      requestId: crypto.randomUUID(),
      documentId,
    });
    return result.ok ? result.data : null;
  } catch {
    return null;
  }
}

export function useReadingPosition(
  api: ReaderApi,
  initial: ReadingPositionDto,
) {
  const view = useRef({ documentId: initial.documentId, generation: 0 });
  const sequence = useRef(0);
  const [status, setStatus] = useState<"idle" | "saving" | "saved" | "error">(
    "idle",
  );
  const [error, setError] = useState<string | null>(null);
  const [recovery, setRecovery] = useState<PositionRecovery | null>(
    () => queues.get(initial.documentId)?.recovery ?? null,
  );

  useEffect(() => {
    if (view.current.documentId === initial.documentId) return;
    view.current = {
      documentId: initial.documentId,
      generation: view.current.generation + 1,
    };
    sequence.current = 0;
    setStatus("idle");
    setError(null);
    setRecovery(queues.get(initial.documentId)?.recovery ?? null);
  }, [initial.documentId]);

  useEffect(() => {
    const entry = entryFor(initial.documentId, initial.revision);
    const update = () => setRecovery(entry.recovery);
    entry.listeners.add(update);
    update();
    return () => {
      entry.listeners.delete(update);
    };
  }, [initial.documentId, initial.revision]);

  const publishRecovery = useCallback(
    (documentId: string, value: PositionRecovery | null) => {
      const entry = queues.get(documentId);
      if (entry) {
        entry.recovery = value;
        for (const listener of entry.listeners) listener();
      }
    },
    [],
  );

  const publishError = useCallback(
    (
      documentId: string,
      generation: number,
      callSequence: number,
      reason: unknown,
    ) => {
      if (
        view.current.documentId === documentId &&
        view.current.generation === generation &&
        sequence.current === callSequence
      ) {
        setStatus("error");
        setError(errorMessage(reason));
      }
    },
    [],
  );

  const persist = useCallback(
    async (entry: QueueEntry, intent: SaveIntent) => {
      entry.uncertain = intent;
      const result = await api.saveReadingPosition(intent);
      if (!result.ok) {
        if (result.error.code === "Conflict") {
          entry.uncertain = null;
          const durable = await readDurable(api, intent.documentId);
          publishRecovery(intent.documentId, {
            kind: "conflict",
            intent,
            durable,
            message: result.error.message,
          });
        } else {
          publishRecovery(intent.documentId, {
            kind: "uncertain",
            intent,
            message: result.error.message,
          });
        }
        throw new Error(result.error.message);
      }
      entry.revision = result.data.revision;
      entry.uncertain = null;
      return result.data;
    },
    [api, publishRecovery],
  );

  const save = useCallback(
    (pageIndex: number, zoom: number) => {
      const documentId = initial.documentId;
      const generation = view.current.generation;
      const callSequence = ++sequence.current;
      setStatus("saving");
      setError(null);
      const entry = entryFor(documentId, initial.revision);
      entry.local = { pageIndex, zoom };

      const request = entry.tail.then(async () => {
        if (entry.recovery) throw new Error(entry.recovery.message);
        const intent: SaveIntent = {
          requestId: crypto.randomUUID(),
          documentId,
          expectedRevision: entry.revision,
          pageIndex,
          zoom,
        };
        return persist(entry, intent);
      });
      entry.tail = request.then(
        () => undefined,
        () => undefined,
      );
      void request.then(
        () => {
          if (
            view.current.documentId === documentId &&
            view.current.generation === generation &&
            callSequence === sequence.current
          ) {
            setStatus("saved");
            setError(null);
          }
        },
        (reason) => publishError(documentId, generation, callSequence, reason),
      );
      return request;
    },
    [initial.documentId, initial.revision, persist, publishError],
  );

  const retryUncertain = useCallback(() => {
    const documentId = initial.documentId;
    const generation = view.current.generation;
    const callSequence = ++sequence.current;
    const entry = entryFor(documentId, initial.revision);
    const current = entry.recovery;
    if (current?.kind !== "uncertain")
      return Promise.reject(
        new Error("There is no uncertain save to retry."),
      );
    const localAtStart = entry.local;
    setStatus("saving");
    setError(null);
    const request = entry.tail.then(async () => {
      const result = await api.saveReadingPosition(current.intent);
      if (!result.ok) {
        if (result.error.code === "Conflict") {
          entry.uncertain = null;
          const durable = await readDurable(api, documentId);
          publishRecovery(documentId, {
            kind: "conflict",
            intent: current.intent,
            durable,
            message: result.error.message,
          });
        }
        throw new Error(result.error.message);
      }
      entry.revision = result.data.revision;
      entry.uncertain = null;
      const newerLocal =
        entry.local !== localAtStart ||
        (entry.local !== null &&
          (entry.local.pageIndex !== current.intent.pageIndex ||
            entry.local.zoom !== current.intent.zoom));
      if (newerLocal) {
        publishRecovery(documentId, {
          kind: "conflict",
          intent: current.intent,
          durable: result.data,
          message:
            "The previous save was confirmed, but another local position is pending.",
        });
      } else {
        entry.local = null;
        publishRecovery(documentId, null);
      }
      return result.data;
    });
    entry.tail = request.then(
      () => undefined,
      () => undefined,
    );
    void request.then(
      () => {
        if (
          view.current.documentId === documentId &&
          view.current.generation === generation &&
          callSequence === sequence.current
        ) {
          if (entry.recovery) {
            setStatus("error");
            setError(entry.recovery.message);
          } else {
            setStatus("saved");
            setError(null);
          }
        }
      },
      (reason) => publishError(documentId, generation, callSequence, reason),
    );
    return request;
  }, [
    api,
    initial.documentId,
    initial.revision,
    publishError,
    publishRecovery,
  ]);

  const refreshConflict = useCallback(async () => {
    const documentId = initial.documentId;
    const entry = entryFor(documentId, initial.revision);
    const current = entry.recovery;
    if (current?.kind !== "conflict") return null;
    const durable = await readDurable(api, documentId);
    if (durable) {
      entry.revision = durable.revision;
      const updated = { ...current, durable };
      publishRecovery(documentId, updated);
      return durable;
    }
    return null;
  }, [api, initial.documentId, initial.revision, publishRecovery]);

  const resolveConflict = useCallback(
    (choice: "keep-saved" | "save-local") => {
      const documentId = initial.documentId;
      const generation = view.current.generation;
      const callSequence = ++sequence.current;
      const entry = entryFor(documentId, initial.revision);
      const current = entry.recovery;
      if (current?.kind !== "conflict")
        return Promise.reject(new Error("No hay un conflicto que resolver."));
      setStatus("saving");
      setError(null);
      const request = entry.tail.then(async () => {
        let durable = current.durable;
        if (!durable) durable = await readDurable(api, documentId);
        if (!durable)
          throw new Error("Could not retrieve the saved position.");
        entry.revision = durable.revision;
        if (choice === "keep-saved") {
          entry.local = null;
          entry.uncertain = null;
          publishRecovery(documentId, null);
          return durable;
        }
        const local = entry.local ?? {
          pageIndex: current.intent.pageIndex,
          zoom: current.intent.zoom,
        };
        const saved = await persist(entry, {
          requestId: crypto.randomUUID(),
          documentId,
          expectedRevision: durable.revision,
          pageIndex: local.pageIndex,
          zoom: local.zoom,
        });
        if (entry.local === local) entry.local = null;
        if (entry.recovery === current) publishRecovery(documentId, null);
        return saved;
      });
      entry.tail = request.then(
        () => undefined,
        () => undefined,
      );
      void request.then(
        () => {
          if (
            view.current.documentId === documentId &&
            view.current.generation === generation &&
            callSequence === sequence.current
          ) {
            setStatus("saved");
            setError(null);
          }
        },
        (reason) => publishError(documentId, generation, callSequence, reason),
      );
      return request;
    },
    [
      api,
      initial.documentId,
      initial.revision,
      persist,
      publishError,
      publishRecovery,
    ],
  );

  return {
    save,
    status,
    error,
    recovery,
    retryUncertain,
    refreshConflict,
    resolveConflict,
  };
}
