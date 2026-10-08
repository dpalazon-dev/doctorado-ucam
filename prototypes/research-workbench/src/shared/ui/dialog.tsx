// Incorporated shadcn/ui-style accessible dialog built on Radix primitives (MIT).
import * as DialogPrimitive from "@radix-ui/react-dialog";
import type { ComponentProps } from "react";
export const Dialog = DialogPrimitive.Root;
export const DialogTitle = DialogPrimitive.Title;
export const DialogDescription = DialogPrimitive.Description;
type DialogContentProps = ComponentProps<typeof DialogPrimitive.Content> & {
  showClose?: boolean;
};
export function DialogContent({
  children,
  showClose = true,
  ...props
}: DialogContentProps) {
  return (
    <DialogPrimitive.Portal>
      <DialogPrimitive.Overlay className="dialog-overlay" />
      <DialogPrimitive.Content className="dialog-content" {...props}>
        {children}
        {showClose && (
          <DialogPrimitive.Close aria-label="Close" className="dialog-close">
            ×
          </DialogPrimitive.Close>
        )}
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  );
}
