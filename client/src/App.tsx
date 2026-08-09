import { useState } from "react";
import { motion } from "motion/react";
import { AlertTriangle, Loader2, PlayCircle } from "lucide-react";
import { PreflightApiError, runPreflight, type Report } from "@/api";
import { UploadZone } from "@/components/UploadZone";
import { ScoreGauge } from "@/components/ScoreGauge";
import { SummaryGrid } from "@/components/SummaryGrid";
import { TransactionList } from "@/components/TransactionList";
import { ThemeToggle } from "@/components/theme-toggle";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";

const DEMO_VIDEO_URL = "https://drive.google.com/file/d/1_cn-ZG8U8fIvYCVIVE8zCn18LTXAc9rz/view?usp=drive_link";

const fadeUp = {
  hidden: { opacity: 0, y: 10 },
  show: { opacity: 1, y: 0 },
};

function App() {
  const [oldFile, setOldFile] = useState<File | null>(null);
  const [newFile, setNewFile] = useState<File | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [report, setReport] = useState<Report | null>(null);

  const canRun = oldFile !== null && newFile !== null && !isLoading;

  async function handleRun() {
    if (!oldFile || !newFile) return;
    setIsLoading(true);
    setError(null);
    try {
      setReport(await runPreflight(oldFile, newFile));
    } catch (err) {
      setError(err instanceof PreflightApiError ? err.message : "Something went wrong replaying these programs.");
      setReport(null);
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className="mx-auto flex min-h-svh max-w-3xl flex-col gap-6 px-4 py-10 sm:py-14">
      <motion.header
        initial="hidden"
        animate="show"
        variants={fadeUp}
        transition={{ duration: 0.35 }}
        className="flex items-start justify-between gap-4"
      >
        <div className="flex flex-col gap-2">
          <div className="flex items-center gap-2.5">
            <img src="/logo.png" alt="Preflight logo" className="size-8" />
            <h1 className="text-3xl font-semibold tracking-tight text-foreground">Preflight</h1>
          </div>
          <p className="max-w-md text-sm text-muted-foreground">
            Replay the same transactions against two builds of a Solana program and see exactly
            what changed before you upgrade.
          </p>
        </div>
        <ThemeToggle />
      </motion.header>

      <motion.div
        initial="hidden"
        animate="show"
        variants={fadeUp}
        transition={{ duration: 0.35, delay: 0.02 }}
        className="flex justify-center"
      >
        <Button asChild size="lg" className="h-12 px-8 text-base">
          <a href={DEMO_VIDEO_URL} target="_blank" rel="noopener noreferrer">
            <PlayCircle className="size-5" />
            Watch Demo
          </a>
        </Button>
      </motion.div>

      <motion.div initial="hidden" animate="show" variants={fadeUp} transition={{ duration: 0.35, delay: 0.05 }}>
        <Card className="gap-5 p-6">
          <div className="grid gap-4 sm:grid-cols-2">
            <UploadZone
              label="Old program"
              hint="The currently deployed build"
              file={oldFile}
              onFileSelected={setOldFile}
              disabled={isLoading}
            />
            <UploadZone
              label="New program"
              hint="The candidate upgrade"
              file={newFile}
              onFileSelected={setNewFile}
              disabled={isLoading}
            />
          </div>

          <div className="flex flex-wrap items-center justify-center gap-3">
            <Button size="lg" disabled={!canRun} onClick={handleRun}>
              {isLoading && <Loader2 className="size-4 animate-spin" />}
              {isLoading ? "Replaying transactions…" : "Run Preflight"}
            </Button>
          </div>

          <p className="text-center text-xs text-balance text-muted-foreground">
            Preflight replays a real transaction sequence against both builds and shows you
            exactly what changed — new failures, altered account state, and compute cost
            differences — before you deploy.
          </p>
        </Card>
      </motion.div>

      {error && (
        <motion.div initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }}>
          <Alert variant="destructive">
            <AlertTriangle />
            <AlertTitle>Preflight couldn't complete this run</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        </motion.div>
      )}

      {report && (
        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.35 }}
          className="flex flex-col gap-6"
        >
          <Card className="flex-row flex-wrap items-center justify-between gap-6 p-6">
            <div className="flex flex-col gap-1.5 font-mono text-xs">
              <div>
                <span className="text-muted-foreground">old:</span>{" "}
                <span className="text-foreground">{report.old_program}</span>
              </div>
              <div>
                <span className="text-muted-foreground">new:</span>{" "}
                <span className="text-foreground">{report.new_program}</span>
              </div>
            </div>
            <ScoreGauge score={report.summary.safety_score} />
          </Card>

          <SummaryGrid summary={report.summary} />

          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold text-foreground">Transactions</h2>
            <TransactionList comparisons={report.comparisons} />
          </div>
        </motion.div>
      )}
    </div>
  );
}

export default App;
