import { useEffect, useState } from "react";
import { animate, motion, useMotionValue } from "motion/react";

/** Mirrors the thresholds in crates/report/src/lib.rs::verdict. */
function verdict(score: number): {
  text: string;
  tier: "safe" | "low" | "elevated" | "high";
} {
  if (score >= 90) {
    return { text: "Safe to deploy. No meaningful behavioral differences detected.", tier: "safe" };
  }
  if (score >= 70) {
    return { text: "Low risk. Minor differences detected — review recommended before deploying.", tier: "low" };
  }
  if (score >= 40) {
    return { text: "Elevated risk. Behavioral changes detected — review carefully before deploying.", tier: "elevated" };
  }
  return { text: "High risk. Regressions detected. Do not deploy without further review.", tier: "high" };
}

const TIER_COLOR: Record<string, string> = {
  safe: "var(--success)",
  low: "var(--info)",
  elevated: "var(--warning)",
  high: "var(--destructive)",
};

const RADIUS = 52;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function ScoreGauge({ score }: { score: number }) {
  const { text, tier } = verdict(score);
  const color = TIER_COLOR[tier];

  const motionScore = useMotionValue(0);
  const [display, setDisplay] = useState(0);

  useEffect(() => {
    const controls = animate(motionScore, score, { duration: 1, ease: "easeOut" });
    const unsubscribe = motionScore.on("change", (value) => setDisplay(Math.round(value)));
    return () => {
      controls.stop();
      unsubscribe();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [score]);

  return (
    <div className="flex flex-col items-center gap-3">
      <div className="relative size-32 shrink-0">
        <svg viewBox="0 0 120 120" className="size-32 -rotate-90">
          <circle cx="60" cy="60" r={RADIUS} fill="none" stroke="var(--border)" strokeWidth="8" />
          <motion.circle
            cx="60"
            cy="60"
            r={RADIUS}
            fill="none"
            stroke={color}
            strokeWidth="8"
            strokeLinecap="round"
            strokeDasharray={CIRCUMFERENCE}
            initial={{ strokeDashoffset: CIRCUMFERENCE }}
            animate={{ strokeDashoffset: CIRCUMFERENCE * (1 - score / 100) }}
            transition={{ duration: 1, ease: "easeOut" }}
          />
        </svg>
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <span className="font-mono text-3xl font-semibold tabular-nums text-foreground">{display}</span>
          <span className="text-[11px] text-muted-foreground">/ 100</span>
        </div>
      </div>
      <p className="max-w-56 text-center text-sm text-muted-foreground">{text}</p>
    </div>
  );
}
