"use client";

import { useState, useCallback, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Shield,
  Lock,
  TrendingUp,
  Activity,
  ArrowRightLeft,
  CheckCircle,
  Loader2,
  RefreshCw,
} from "lucide-react";

function Particle({
  id,
  x,
  y,
  targetX,
  targetY,
  matched,
  onDone,
}: {
  id: number;
  x: number;
  y: number;
  targetX: number;
  targetY: number;
  matched: boolean;
  onDone: () => void;
}) {
  return (
    <motion.div
      className={`absolute w-3 h-3 rounded-full ${matched ? "bg-emerald-400" : "bg-indigo-400/60"}`}
      style={{ left: x, top: y }}
      initial={{ scale: 0 }}
      animate={{
        scale: matched ? [0, 1.2, 0] : 1,
        x: matched ? targetX - x : 0,
        y: matched ? targetY - y : 0,
        opacity: matched ? [1, 1, 0] : [0, 1],
      }}
      transition={{
        duration: matched ? 1.5 : 0.5,
        ease: matched ? "easeInOut" : "backOut",
      }}
      onAnimationComplete={() => {
        if (matched) onDone();
      }}
    />
  );
}

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

export default function DarkPoolDemoPage() {
  const [amount, setAmount] = useState("");
  const [price, setPrice] = useState("");
  const [hideDetails, setHideDetails] = useState(true);
  const [proofState, setProofState] = useState<"idle" | "placing" | "placed" | "matched">("idle");
  const [matchId, setMatchId] = useState(0);
  const [particles, setParticles] = useState<
    Array<{ id: number; x: number; y: number; targetX: number; targetY: number }>
  >([]);

  const totalLiquidity = 125000;
  const matchProbability = useMemo(() => {
    if (!amount) return 0;
    const a = parseFloat(amount);
    if (isNaN(a)) return 0;
    return Math.min(85, Math.round(((a / totalLiquidity) * 100 + 60) * 10) / 10);
  }, [amount]);

  const placeOrder = useCallback(() => {
    if (!amount || !price) return;
    setProofState("placing");
    setTimeout(() => {
      setProofState("placed");

      // Spawn particles
      const poolRect = document.getElementById("pool-area")?.getBoundingClientRect();
      if (poolRect) {
        const cx = poolRect.width / 2;
        const cy = poolRect.height / 2;
        const newParticles = Array.from({ length: 6 }, (_, i) => ({
          id: Date.now() + i,
          x: Math.random() * poolRect.width * 0.3 + poolRect.width * 0.1,
          y: Math.random() * poolRect.height * 0.6 + poolRect.height * 0.2,
          targetX: cx,
          targetY: cy,
        }));
        setParticles((prev) => [...prev.slice(-20), ...newParticles]);
      }

      // Simulate match after delay
      setTimeout(() => {
        setProofState("matched");
        setMatchId((prev) => prev + 1);
        if (poolRect) {
          const cx = poolRect.width / 2;
          const cy = poolRect.height / 2;
          const sparkParticles = Array.from({ length: 12 }, (_, i) => ({
            id: Date.now() + 100 + i,
            x: cx + (Math.random() - 0.5) * 60,
            y: cy + (Math.random() - 0.5) * 60,
            targetX: cx + (Math.random() - 0.5) * 120,
            targetY: cy + (Math.random() - 0.5) * 120,
          }));
          setParticles((prev) => [...prev.slice(-30), ...sparkParticles]);
        }
      }, 3500);
    }, 1500);
  }, [amount, price]);

  const reset = useCallback(() => {
    setProofState("idle");
    setParticles([]);
    setMatchId(0);
    setAmount("");
    setPrice("");
    setHideDetails(true);
  }, []);

  return (
    <div className="min-h-screen flex flex-col items-center justify-center px-4 py-24">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-center mb-8"
      >
        <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-indigo-400 via-purple-400 to-cyan-400 bg-clip-text text-transparent">
          Privacy Dark Pool
        </h1>
        <p className="mt-2 text-white/50 text-sm">Anonymous order matching with zero-knowledge proofs</p>
      </motion.div>

      <div className="w-full max-w-5xl grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* Left Panel - Your Order */}
        <Card className="border-neutral-800 bg-neutral-900/50 backdrop-blur">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-white">
              <Lock className="size-4 text-indigo-400" />
              Your Order
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Amount Input */}
            <div>
              <label className="text-xs text-white/40 font-mono">Amount (SOL)</label>
              <motion.div
                className="relative mt-1"
                animate={{ filter: hideDetails ? "blur(6px)" : "blur(0px)" }}
                transition={{ duration: 0.3 }}
              >
                <input
                  type="number"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="0.00"
                  disabled={proofState !== "idle"}
                  className="w-full h-10 rounded-lg border border-neutral-700 bg-neutral-800 px-3 text-white text-sm focus:outline-none focus:border-indigo-500 transition-colors disabled:opacity-50"
                />
              </motion.div>
            </div>

            {/* Price Input */}
            <div>
              <label className="text-xs text-white/40 font-mono">Limit Price (USD)</label>
              <motion.div
                className="relative mt-1"
                animate={{ filter: hideDetails ? "blur(6px)" : "blur(0px)" }}
                transition={{ duration: 0.3 }}
              >
                <input
                  type="number"
                  value={price}
                  onChange={(e) => setPrice(e.target.value)}
                  placeholder="0.00"
                  disabled={proofState !== "idle"}
                  className="w-full h-10 rounded-lg border border-neutral-700 bg-neutral-800 px-3 text-white text-sm focus:outline-none focus:border-indigo-500 transition-colors disabled:opacity-50"
                />
              </motion.div>
            </div>

            {/* Privacy toggle */}
            <div className="flex items-center justify-between">
              <span className="text-xs text-white/40">Privacy Mode</span>
              <button
                onClick={() => setHideDetails(!hideDetails)}
                className="text-xs text-indigo-400 hover:text-indigo-300 transition-colors"
              >
                {hideDetails ? "Reveal Values" : "Hide Values"}
              </button>
            </div>

            {/* Hidden order summary */}
            <AnimatePresence>
              {hideDetails && amount && price && (
                <motion.div
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: "auto" }}
                  exit={{ opacity: 0, height: 0 }}
                  className="text-xs text-white/30 font-mono space-y-1"
                >
                  <p>Amount: {"•".repeat(amount.length || 4)}</p>
                  <p>Price: {"•".repeat(price.length || 4)}</p>
                  <p>Privacy: Maximum</p>
                </motion.div>
              )}
            </AnimatePresence>

            {/* Action Buttons */}
            <div className="flex flex-col gap-2 pt-2">
              {proofState === "idle" && (
                <Button
                  onClick={placeOrder}
                  disabled={!amount || !price}
                  className="w-full bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white"
                >
                  <Shield className="size-4 mr-1" />
                  Place Order
                </Button>
              )}

              {proofState === "placing" && (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="flex flex-col items-center py-3"
                >
                  <RotatingRings />
                  <p className="mt-2 text-xs text-indigo-300 font-mono">
                    Generating ZK proof...
                  </p>
                </motion.div>
              )}

              {proofState === "placed" && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.5 }}
                  animate={{ opacity: 1, scale: 1 }}
                  className="flex flex-col items-center py-3 gap-2"
                >
                  <Badge className="bg-indigo-600 text-white">
                    Order Placed — Searching Match
                  </Badge>
                  <Loader2 className="size-4 animate-spin text-indigo-400" />
                </motion.div>
              )}

              {proofState === "matched" && (
                <motion.div
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ type: "spring", stiffness: 300, damping: 20 }}
                >
                  <Badge className="w-full justify-center bg-emerald-600 text-white py-2 text-sm">
                    <CheckCircle className="size-4 mr-1" />
                    Match Found!
                  </Badge>
                </motion.div>
              )}
            </div>

            {/* Reset */}
            {proofState !== "idle" && (
              <div className="text-center">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={reset}
                  className="text-white/30 hover:text-white"
                >
                  <RefreshCw className="size-3 mr-1" />
                  New Order
                </Button>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Center - Pool Visualization */}
        <Card className="border-neutral-800 bg-neutral-900/50 backdrop-blur">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-white">
              <Activity className="size-4 text-cyan-400" />
              Dark Pool
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div
              id="pool-area"
              className="relative w-full aspect-square rounded-xl border border-neutral-800 bg-neutral-950 overflow-hidden"
            >
              {/* Central area effect */}
              <div className="absolute inset-0 bg-gradient-radial from-indigo-900/20 via-transparent to-transparent" />

              {/* Match counter */}
              <AnimatePresence>
                {matchId > 0 && (
                  <motion.div
                    key={matchId}
                    initial={{ opacity: 0, scale: 0 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0 }}
                    className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-10"
                  >
                    <motion.div
                      initial={{ scale: 0 }}
                      animate={{ scale: [0, 1.5, 1] }}
                      transition={{ type: "spring", stiffness: 300, damping: 15 }}
                      className="flex flex-col items-center"
                    >
                      <ArrowRightLeft className="size-8 text-emerald-400" />
                      <span className="text-emerald-400 text-xs font-mono mt-1">MATCHED</span>
                    </motion.div>
                  </motion.div>
                )}
              </AnimatePresence>

              {/* Floating particles */}
              {particles.map((p) => (
                <Particle
                  key={p.id}
                  {...p}
                  matched={proofState === "matched" && p.id >= particles[particles.length - 12]?.id}
                  onDone={() => {}}
                />
              ))}

              {/* Anonymized orders floating */}
              {proofState === "placed" || proofState === "matched" ? (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="absolute bottom-4 left-1/2 -translate-x-1/2"
                >
                  <div className="flex items-center gap-1 px-2 py-1 rounded-full bg-indigo-500/10 border border-indigo-500/20">
                    <Lock className="size-3 text-indigo-400" />
                    <span className="text-xs text-indigo-400 font-mono">ANON_ORDER</span>
                  </div>
                </motion.div>
              ) : (
                <motion.p
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 0.4 }}
                  className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-xs text-white/30 font-mono"
                >
                  Pool Active
                </motion.p>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Right Panel - Pool Status */}
        <Card className="border-neutral-800 bg-neutral-900/50 backdrop-blur">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-white">
              <TrendingUp className="size-4 text-emerald-400" />
              Pool Status
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-white/40">Total Liquidity</span>
                <span className="text-white font-mono">{totalLiquidity.toLocaleString()} SOL</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-white/40">Open Orders</span>
                <span className="text-white font-mono">42</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-white/40">Match Probability</span>
                <span className="text-indigo-400 font-mono">{matchProbability}%</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-white/40">24h Volume</span>
                <span className="text-white font-mono">1,240 SOL</span>
              </div>
            </div>

            {/* Stats bar */}
            <div className="border-t border-neutral-800 pt-3 space-y-2">
              <div className="flex items-center gap-2 text-xs text-emerald-400 font-mono">
                <Shield className="size-3" />
                Privacy Level: Maximum
              </div>
              <div className="flex items-center gap-2 text-xs text-cyan-400 font-mono">
                <Activity className="size-3" />
                Slippage: 0%
              </div>
              <div className="flex items-center gap-2 text-xs text-purple-400 font-mono">
                <CheckCircle className="size-3" />
                Proof:{" "}
                {proofState === "idle" ? (
                  "Waiting"
                ) : proofState === "placing" ? (
                  "Generating..."
                ) : proofState === "placed" ? (
                  "Verified"
                ) : (
                  "Verified"
                )}
              </div>
            </div>

            <div className="border-t border-neutral-800 pt-3">
              <p className="text-xs text-white/30 leading-relaxed">
                Orders are hidden from the public mempool. Zero-knowledge proofs ensure fair matching without revealing trade details.
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
