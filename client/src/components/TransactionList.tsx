import { useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { ChevronDown } from "lucide-react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import type { TxComparison, TxExecutionResult, TxOutcomeCategory } from "@/api";

const CATEGORY_META: Record<TxOutcomeCategory, { label: string; className: string }> = {
  Unchanged: { label: "unchanged", className: "bg-muted text-muted-foreground" },
  ComputeUnitsChanged: { label: "compute units changed", className: "bg-info/10 text-info" },
  BehaviorChanged: { label: "behavior changed", className: "bg-warning/10 text-warning" },
  ErrorChanged: { label: "error changed", className: "bg-warning/10 text-warning" },
  NewFailure: { label: "new failure", className: "bg-destructive/10 text-destructive" },
  NewSuccess: { label: "new success", className: "bg-destructive/10 text-destructive" },
};

const container = {
  hidden: {},
  show: { transition: { staggerChildren: 0.035 } },
};

const item = {
  hidden: { opacity: 0, y: 8 },
  show: { opacity: 1, y: 0 },
};

export function TransactionList({ comparisons }: { comparisons: TxComparison[] }) {
  return (
    <motion.ul variants={container} initial="hidden" animate="show" className="flex flex-col gap-2">
      {comparisons.map((comparison) => (
        <motion.li key={comparison.label} variants={item}>
          <TransactionRow comparison={comparison} />
        </motion.li>
      ))}
    </motion.ul>
  );
}

function TransactionRow({ comparison }: { comparison: TxComparison }) {
  const [open, setOpen] = useState(false);
  const meta = CATEGORY_META[comparison.category];

  return (
    <Collapsible open={open} onOpenChange={setOpen} className="rounded-lg border border-border bg-card">
      <CollapsibleTrigger className="flex w-full items-center gap-3 px-4 py-3 text-left">
        <Badge className={cn("shrink-0 uppercase tracking-wide", meta.className)}>{meta.label}</Badge>
        <span className="truncate font-mono text-sm font-medium text-foreground">{comparison.label}</span>
        <span className="ml-auto flex shrink-0 items-center gap-2 font-mono text-xs text-muted-foreground">
          {comparison.old.compute_units_consumed} → {comparison.new.compute_units_consumed} CU
          <span className="opacity-70">
            ({comparison.compute_units_delta >= 0 ? "+" : ""}
            {comparison.compute_units_delta})
          </span>
          <ChevronDown className={cn("size-4 transition-transform", open && "rotate-180")} />
        </span>
      </CollapsibleTrigger>
      <AnimatePresence initial={false}>
        {open && (
          <CollapsibleContent asChild forceMount>
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={{ duration: 0.2, ease: "easeInOut" }}
              className="overflow-hidden"
            >
              <div className="flex flex-col gap-3 border-t border-border px-4 pt-3 pb-4">
                <p className="text-sm text-muted-foreground">{comparison.description}</p>
                <p className="text-sm font-medium text-foreground">{comparison.notes}</p>
                <div className="grid gap-3 sm:grid-cols-2">
                  <ExecutionColumn title="Old program" result={comparison.old} />
                  <ExecutionColumn title="New program" result={comparison.new} />
                </div>
              </div>
            </motion.div>
          </CollapsibleContent>
        )}
      </AnimatePresence>
    </Collapsible>
  );
}

function ExecutionColumn({ title, result }: { title: string; result: TxExecutionResult }) {
  return (
    <div className="flex flex-col gap-1.5 rounded-md border border-border bg-background p-3">
      <h4 className="text-xs font-semibold text-muted-foreground">
        {title} —{" "}
        <span className={result.success ? "text-success" : "text-destructive"}>
          {result.success ? "succeeded" : "failed"}
        </span>
      </h4>
      {result.error && <p className="font-mono text-xs text-destructive">{result.error}</p>}
      {result.counter_state && (
        <p className="text-xs text-foreground">
          value = <code className="font-mono">{result.counter_state.value}</code>
        </p>
      )}
      {result.logs.length > 0 && (
        <pre className="max-h-44 overflow-auto rounded bg-muted p-2 font-mono text-[11px] leading-relaxed whitespace-pre-wrap text-muted-foreground">
          {result.logs.join("\n")}
        </pre>
      )}
    </div>
  );
}
