"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Shield,
  CheckCircle,
  VoteIcon,
  ThumbsUp,
  ThumbsDown,
  Minus,
  RefreshCw,
  Lock,
} from "lucide-react";

type VoteOption = "yes" | "no" | "abstain";
type VoteState = "idle" | "proving" | "voted";

interface ParticleData {
  id: number;
  x: number;
  y: number;
  color: string;
}

const optionConfig: Record<
  VoteOption,
  {
    label: string;
    icon: typeof ThumbsUp;
    color: string;
    bgClass: string;
    hoverClass: string;
    particleColor: string;
  }
> = {
  yes: {
    label: "Yes",
    icon: ThumbsUp,
    color: "text-emerald-400",
    bgClass: "bg-emerald-500/20 border-emerald-500/30",
    hoverClass: "hover:bg-emerald-500/30",
    particleColor: "#34d399",
  },
  no: {
    label: "No",
    icon: ThumbsDown,
    color: "text-red-400",
    bgClass: "bg-red-500/20 border-red-500/30",
    hoverClass: "hover:bg-red-500/30",
    particleColor: "#f87171",
  },
  abstain: {
    label: "Abstain",
    icon: Minus,
    color: "text-neutral-400",
    bgClass: "bg-neutral-500/20 border-neutral-500/30",
    hoverClass: "hover:bg-neutral-500/30",
    particleColor: "#a3a3a3",
  },
};

const resultsConfig: Record<VoteOption, { label: string; color: string; barClass: string }> = {
  yes: { label: "Yes", color: "text-emerald-400", barClass: "bg-emerald-500" },
  no: { label: "No", color: "text-red-400", barClass: "bg-red-500" },
  abstain: { label: "Abstain", color: "text-neutral-400", barClass: "bg-neutral-500" },
};

function RotatingRings() {
  return (
    <div className="relative w-16 h-16">
      <motion.div
        className="absolute inset-0 rounded-full border-2 border-transparent border-t-indigo-400"
        animate={{ rotate: 360 }}
        transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
      />
      <motion.div
        className="absolute inset-2 rounded-full border-2 border-transparent border-t-purple-400"
        animate={{ rotate: -360 }}
        transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }}
      />
      <div className="absolute inset-0 flex items-center justify-center">
        <Shield className="size-4 text-white/60" />
      </div>
    </div>
  );
}

function VoteParticle({ data, onDone }: { data: ParticleData; onDone: () => void }) {
  return (
    <motion.div
      className="fixed z-50 w-3 h-3 rounded-full pointer-events-none"
      style={{
        left: data.x - 6,
        top: data.y - 6,
        backgroundColor: data.color,
      }}
      initial={{ scale: 1.5, opacity: 1 }}
      animate={{
        scale: 0,
        opacity: 0,
        x: (Math.random() - 0.5) * 60,
        y: -(Math.random() * 80 + 40),
      }}
      transition={{ duration: 0.7, ease: "easeOut" }}
      onAnimationComplete={onDone}
    />
  );
}

export default function VotingDemoPage() {
  const [voteState, setVoteState] = useState<VoteState>("idle");
  const [selectedOption, setSelectedOption] = useState<VoteOption | null>(null);
  const [results, setResults] = useState({ yes: 12, no: 5, abstain: 3 });
  const [particles, setParticles] = useState<ParticleData[]>([]);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const buttonRefs = useRef<Record<VoteOption, HTMLButtonElement | null>>({
    yes: null,
    no: null,
    abstain: null,
  });

  const totalVotes = results.yes + results.no + results.abstain;

  const getPercentage = (votes: number) => {
    if (totalVotes === 0) return 0;
    return Math.round((votes / totalVotes) * 100);
  };

  const castVote = useCallback(
    (option: VoteOption) => {
      if (voteState !== "idle") return;
      setSelectedOption(option);
      setVoteState("proving");

      setTimeout(() => {
        setVoteState("voted");
        setResults((prev) => ({ ...prev, [option]: prev[option] + 1 }));

        // Spawn particles from the clicked button
        const btn = buttonRefs.current[option];
        if (btn) {
          const rect = btn.getBoundingClientRect();
          const newParticle: ParticleData = {
            id: Date.now(),
            x: rect.left + rect.width / 2,
            y: rect.top + rect.height / 2,
            color: optionConfig[option].particleColor,
          };
          setParticles((prev) => [...prev, newParticle]);
        }
      }, 2000);
    },
    [voteState]
  );

  const removeParticle = useCallback((id: number) => {
    setParticles((prev) => prev.filter((p) => p.id !== id));
  }, []);

  const reset = useCallback(() => {
    setVoteState("idle");
    setSelectedOption(null);
    setResults({ yes: 12, no: 5, abstain: 3 });
    setParticles([]);
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  // Simulate incoming votes
  useEffect(() => {
    if (voteState !== "idle") {
      intervalRef.current = setInterval(() => {
        const options: VoteOption[] = ["yes", "no", "abstain"];
        const randomOption = options[Math.floor(Math.random() * 3)];
        setResults((prev) => ({ ...prev, [randomOption]: prev[randomOption] + 1 }));
      }, 3000);
    }
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [voteState]);

  return (
    <div className="min-h-screen flex flex-col items-center justify-center px-4 py-24">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-center mb-8"
      >
        <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-indigo-400 via-purple-400 to-cyan-400 bg-clip-text text-transparent">
          Anonymous Voting
        </h1>
        <p className="mt-2 text-white/50 text-sm">Private, verifiable governance with zero-knowledge proofs</p>
      </motion.div>

      <div className="w-full max-w-2xl space-y-4">
        {/* Proposal */}
        <Card className="border-neutral-800 bg-neutral-900/50 backdrop-blur">
          <CardHeader>
            <CardTitle className="text-white text-lg">Proposal</CardTitle>
          </CardHeader>
          <CardContent>
            <h2 className="text-xl font-semibold text-white mb-2">
              Should Privy SVM launch on Mainnet?
            </h2>
            <p className="text-sm text-white/50 leading-relaxed">
              This proposal seeks community consensus on deploying the Privy SVM zero-knowledge
              privacy protocol on Solana Mainnet. Voting is private — only aggregate results are
              visible on-chain through ZK proofs.
            </p>
            <div className="mt-3 flex items-center gap-2">
              <Badge variant="outline" className="border-indigo-600/30 text-indigo-400">
                Governance
              </Badge>
              <Badge variant="outline" className="border-purple-600/30 text-purple-400">
                Quorum: 20 votes
              </Badge>
              <Badge variant="outline" className="border-emerald-600/30 text-emerald-400">
                Active
              </Badge>
            </div>
          </CardContent>
        </Card>

        {/* Voting Options */}
        <Card className="border-neutral-800 bg-neutral-900/50 backdrop-blur">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-white">
              <VoteIcon className="size-4 text-indigo-400" />
              Cast Your Vote
            </CardTitle>
          </CardHeader>
          <CardContent>
            {/* Vote State */}
            <AnimatePresence mode="wait">
              {voteState === "idle" && (
                <motion.div
                  key="idle"
                  exit={{ opacity: 0, y: -10 }}
                  className="flex flex-col sm:flex-row gap-3"
                >
                  {(Object.keys(optionConfig) as VoteOption[]).map((option) => {
                    const config = optionConfig[option];
                    const Icon = config.icon;
                    return (
                      <motion.button
                        key={option}
                        ref={(el) => {
                          buttonRefs.current[option] = el;
                        }}
                        onClick={() => castVote(option)}
                        whileHover={{ scale: 1.03 }}
                        whileTap={{ scale: 0.97 }}
                        className={`flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-xl border text-sm font-medium transition-colors ${config.bgClass} ${config.hoverClass} ${config.color}`}
                      >
                        <Icon className="size-4" />
                        {config.label}
                      </motion.button>
                    );
                  })}
                </motion.div>
              )}

              {voteState === "proving" && (
                <motion.div
                  key="proving"
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0 }}
                  className="flex flex-col items-center justify-center py-6"
                >
                  <RotatingRings />
                  <AnimatePresence>
                    {selectedOption && (
                      <motion.p
                        key={selectedOption}
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        className="mt-3 text-sm font-mono"
                      >
                        <span className={optionConfig[selectedOption].color}>
                          Verifying eligibility: {optionConfig[selectedOption].label}
                        </span>
                      </motion.p>
                    )}
                  </AnimatePresence>
                  <p className="mt-1 text-xs text-white/30 font-mono">
                    Generating zero-knowledge proof...
                  </p>
                </motion.div>
              )}

              {voteState === "voted" && (
                <motion.div
                  key="voted"
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  className="flex flex-col items-center justify-center py-6 gap-3"
                >
                  <motion.div
                    initial={{ scale: 0 }}
                    animate={{ scale: [0, 1.3, 1] }}
                    transition={{ type: "spring", stiffness: 400, damping: 15 }}
                  >
                    <CheckCircle className="size-12 text-emerald-400" />
                  </motion.div>
                  <div className="text-center">
                    <p className="text-white font-medium">
                      Vote Cast:{" "}
                      <span className={selectedOption ? optionConfig[selectedOption].color : ""}>
                        {selectedOption ? optionConfig[selectedOption].label : ""}
                      </span>
                    </p>
                    <p className="text-xs text-white/30 mt-1">
                      Your vote is private. Only the aggregate is visible on-chain.
                    </p>
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </CardContent>
        </Card>

        {/* Results */}
        <Card className="border-neutral-800 bg-neutral-900/50 backdrop-blur">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-white">
              <Shield className="size-4 text-emerald-400" />
              Live Results
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {(Object.keys(resultsConfig) as VoteOption[]).map((option) => {
              const config = resultsConfig[option];
              const votes = results[option];
              const percentage = getPercentage(votes);
              return (
                <div key={option}>
                  <div className="flex justify-between text-sm mb-1">
                    <span className={config.color}>{config.label}</span>
                    <span className="text-white font-mono">
                      {votes} votes ({percentage}%)
                    </span>
                  </div>
                  <div className="h-6 rounded-full bg-neutral-800 overflow-hidden">
                    <motion.div
                      className={`h-full rounded-full ${config.barClass}`}
                      initial={{ width: 0 }}
                      animate={{ width: `${percentage}%` }}
                      transition={{
                        type: "spring",
                        stiffness: 100,
                        damping: 20,
                        mass: 0.5,
                      }}
                    />
                  </div>
                </div>
              );
            })}
            <p className="text-xs text-white/30 font-mono text-center">
              Total votes: {totalVotes}
            </p>
          </CardContent>
        </Card>

        {/* Privacy Badge */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.5 }}
          className="flex items-center justify-center gap-3 p-3"
        >
          <Badge
            variant="outline"
            className="border-indigo-600/30 text-indigo-400 gap-1"
          >
            <Lock className="size-3" />
            Zero-Knowledge
          </Badge>
          <Badge
            variant="outline"
            className="border-emerald-600/30 text-emerald-400 gap-1"
          >
            <Shield className="size-3" />
            Publicly Verifiable
          </Badge>
        </motion.div>

        {/* Vote Particles */}
        {particles.map((p) => (
          <VoteParticle key={p.id} data={p} onDone={() => removeParticle(p.id)} />
        ))}

        {/* Reset */}
        <div className="text-center">
          <Button
            variant="ghost"
            size="sm"
            onClick={reset}
            className="text-white/30 hover:text-white"
          >
            <RefreshCw className="size-3 mr-1" />
            Reset Demo
          </Button>
        </div>
      </div>
    </div>
  );
}
