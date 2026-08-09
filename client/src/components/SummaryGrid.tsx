import { motion } from "motion/react";
import { cn } from "@/lib/utils";
import type { RunSummary } from "@/api";

const container = {
  hidden: {},
  show: { transition: { staggerChildren: 0.04 } },
};

const item = {
  hidden: { opacity: 0, y: 8 },
  show: { opacity: 1, y: 0 },
};

export function SummaryGrid({ summary }: { summary: RunSummary }) {
  const cards: { label: string; value: number; tone: "neutral" | "warning" | "destructive" }[] = [
    { label: "Transactions replayed", value: summary.total, tone: "neutral" },
    { label: "Unchanged", value: summary.unchanged, tone: "neutral" },
    { label: "Compute units changed", value: summary.compute_units_changed, tone: "neutral" },
    { label: "Behavior changed", value: summary.behavior_changed, tone: "warning" },
    { label: "Error changed", value: summary.error_changed, tone: "warning" },
    { label: "New failures", value: summary.new_failures, tone: "destructive" },
    { label: "New successes", value: summary.new_successes, tone: "destructive" },
  ];

  return (
    <motion.div
      variants={container}
      initial="hidden"
      animate="show"
      className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4"
    >
      {cards.map((card) => (
        <motion.div
          key={card.label}
          variants={item}
          className="rounded-lg border border-border bg-card p-4 text-center"
        >
          <div
            className={cn(
              "font-mono text-2xl font-semibold tabular-nums",
              card.tone === "neutral" && "text-foreground",
              card.tone === "warning" && "text-warning",
              card.tone === "destructive" && "text-destructive",
            )}
          >
            {card.value}
          </div>
          <div className="mt-1 text-xs text-muted-foreground">{card.label}</div>
        </motion.div>
      ))}
    </motion.div>
  );
}
