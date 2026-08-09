import { useId, useRef, useState } from "react";
import type { DragEvent } from "react";
import { AnimatePresence, motion } from "motion/react";
import { FileCode2, UploadCloud, X } from "lucide-react";
import { cn } from "@/lib/utils";

interface UploadZoneProps {
  label: string;
  hint: string;
  file: File | null;
  onFileSelected: (file: File | null) => void;
  disabled?: boolean;
}

export function UploadZone({ label, hint, file, onFileSelected, disabled }: UploadZoneProps) {
  const inputId = useId();
  const inputRef = useRef<HTMLInputElement>(null);
  const [isDraggedOver, setIsDraggedOver] = useState(false);

  function handleDrop(event: DragEvent<HTMLDivElement>) {
    event.preventDefault();
    setIsDraggedOver(false);
    if (disabled) return;
    const dropped = event.dataTransfer.files[0];
    if (dropped) onFileSelected(dropped);
  }

  return (
    <div className="flex flex-col gap-2">
      <label htmlFor={inputId} className="text-sm font-medium text-foreground">
        {label}
      </label>
      <motion.div
        role="button"
        tabIndex={disabled ? -1 : 0}
        aria-disabled={disabled}
        whileHover={disabled ? undefined : { scale: 1.01 }}
        whileTap={disabled ? undefined : { scale: 0.99 }}
        onKeyDown={(event) => {
          if (!disabled && (event.key === "Enter" || event.key === " ")) {
            event.preventDefault();
            inputRef.current?.click();
          }
        }}
        className={cn(
          "flex min-h-24 cursor-pointer items-center justify-center rounded-lg border-[1.5px] border-dashed border-border bg-card/50 p-5 transition-colors",
          isDraggedOver && "border-primary bg-primary/5",
          file && "border-solid border-success/40 bg-success/5",
          disabled && "cursor-not-allowed opacity-60",
        )}
        onDragOver={(event) => {
          event.preventDefault();
          if (!disabled) setIsDraggedOver(true);
        }}
        onDragLeave={() => setIsDraggedOver(false)}
        onDrop={handleDrop}
        onClick={() => !disabled && inputRef.current?.click()}
      >
        <input
          id={inputId}
          ref={inputRef}
          type="file"
          accept=".so"
          disabled={disabled}
          onChange={(event) => onFileSelected(event.target.files?.[0] ?? null)}
          hidden
        />
        <AnimatePresence mode="wait" initial={false}>
          {file ? (
            <motion.div
              key="file"
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -4 }}
              transition={{ duration: 0.15 }}
              className="flex w-full flex-wrap items-center justify-center gap-2.5 text-sm"
            >
              <FileCode2 className="size-4 shrink-0 text-success" />
              <span className="max-w-40 truncate font-mono font-medium text-foreground sm:max-w-56">
                {file.name}
              </span>
              <span className="text-xs text-muted-foreground">{formatBytes(file.size)}</span>
              <button
                type="button"
                disabled={disabled}
                className="rounded-md p-1 text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive disabled:pointer-events-none"
                onClick={(event) => {
                  event.stopPropagation();
                  onFileSelected(null);
                  if (inputRef.current) inputRef.current.value = "";
                }}
              >
                <X className="size-3.5" />
                <span className="sr-only">Remove file</span>
              </button>
            </motion.div>
          ) : (
            <motion.div
              key="empty"
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -4 }}
              transition={{ duration: 0.15 }}
              className="flex flex-col items-center gap-1.5 text-center"
            >
              <UploadCloud className="size-5 text-muted-foreground" />
              <span className="text-sm text-foreground">Drop a .so file, or click to browse</span>
              <span className="text-xs text-muted-foreground">{hint}</span>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
