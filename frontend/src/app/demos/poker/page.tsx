"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Shield, Zap, Info, RotateCw, Eye, EyeOff, RefreshCw } from "lucide-react";

const SUITS = ["♠", "♥", "♦", "♣"] as const;
const RANKS = ["A", "K", "Q", "J", "10", "9", "8", "7", "6", "5", "4", "3", "2"];

type CardData = { rank: string; suit: string };

function randomCard(): CardData {
  return {
    rank: RANKS[Math.floor(Math.random() * RANKS.length)],
    suit: SUITS[Math.floor(Math.random() * SUITS.length)],
  };
}

function cardColor(suit: string) {
  return suit === "♥" || suit === "♦" ? "text-red-500" : "text-white";
}

function PlayingCard({
  card,
  faceUp,
  index = 0,
}: {
  card: CardData;
  faceUp: boolean;
  index?: number;
}) {
  return (
    <motion.div
      initial={{ rotateY: 90, opacity: 0 }}
      animate={{ rotateY: 0, opacity: 1 }}
      exit={{ rotateY: 90, opacity: 0 }}
      transition={{ type: "spring", stiffness: 200, damping: 20, delay: index * 0.15 }}
      className="relative w-20 h-28 sm:w-24 sm:h-32 rounded-lg shadow-xl"
      style={{ transformStyle: "preserve-3d", perspective: 600 }}
    >
      <AnimatePresence mode="wait">
        {faceUp ? (
          <motion.div
            key={`face-${card.rank}-${card.suit}`}
            initial={{ rotateY: 90 }}
            animate={{ rotateY: 0 }}
            exit={{ rotateY: -90 }}
            transition={{ type: "spring", stiffness: 300, damping: 25 }}
            className={`absolute inset-0 rounded-lg border border-white/10 bg-white flex flex-col items-center justify-center ${cardColor(card.suit)}`}
            style={{ backfaceVisibility: "hidden" }}
          >
            <span className="text-2xl sm:text-3xl font-bold leading-none">{card.rank}</span>
            <span className="text-xl sm:text-2xl">{card.suit}</span>
          </motion.div>
        ) : (
          <motion.div
            key="back"
            initial={{ rotateY: -90 }}
            animate={{ rotateY: 0 }}
            exit={{ rotateY: 90 }}
            transition={{ type: "spring", stiffness: 300, damping: 25 }}
            className="absolute inset-0 rounded-lg border border-white/10 bg-gradient-to-br from-indigo-900 via-purple-900 to-indigo-950 flex flex-col items-center justify-center gap-1"
            style={{ backfaceVisibility: "hidden" }}
          >
            <motion.div
              animate={{ rotate: 360 }}
              transition={{ duration: 8, repeat: Infinity, ease: "linear" }}
            >
              <Shield className="size-6 text-indigo-400/80" />
            </motion.div>
            <span className="text-[10px] font-mono text-indigo-400/60">ZK</span>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}

function RotatingRings() {
  return (
    <div className="relative w-24 h-24">
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
      <motion.div
        className="absolute inset-4 rounded-full border-2 border-transparent border-t-cyan-400"
        animate={{ rotate: 360 }}
        transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
      />
      <div className="absolute inset-0 flex items-center justify-center">
        <Shield className="size-5 text-white/60" />
      </div>
    </div>
  );
}

export default function PokerDemoPage() {
  const [playerHand] = useState<[CardData, CardData]>(() => [randomCard(), randomCard()]);
  const [communityCards] = useState<CardData[]>(() =>
    Array.from({ length: 5 }, () => randomCard())
  );
  const [communityRevealed, setCommunityRevealed] = useState(0);
  const [playerRevealed, setPlayerRevealed] = useState(false);
  const [proofState, setProofState] = useState<"pending" | "generating" | "verified">("pending");

  const generateProof = useCallback(() => {
    if (proofState !== "pending") return;
    setProofState("generating");
    setTimeout(() => setProofState("verified"), 2200);
  }, [proofState]);

  const revealCommunity = useCallback(() => {
    setCommunityRevealed((prev) => Math.min(prev + 1, 5));
  }, []);

  const revealHand = useCallback(() => {
    setPlayerRevealed(true);
  }, []);

  const reset = useCallback(() => {
    setCommunityRevealed(0);
    setPlayerRevealed(false);
    setProofState("pending");
  }, []);

  return (
    <div className="min-h-screen flex flex-col items-center justify-center px-4 py-24">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-center mb-8"
      >
        <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-indigo-400 via-purple-400 to-cyan-400 bg-clip-text text-transparent">
          Zero-Knowledge Poker
        </h1>
        <p className="mt-2 text-white/50 text-sm">Privacy-preserving Texas Hold&apos;em on Solana</p>
      </motion.div>

      <Card className="w-full max-w-4xl border-neutral-800 bg-neutral-900/50 backdrop-blur">
        <CardContent className="p-6 sm:p-8">
          {/* Card Table */}
          <motion.div
            initial={{ scale: 0.95, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            transition={{ delay: 0.3 }}
            className="relative w-full aspect-[2/1] rounded-[2rem] bg-gradient-to-br from-emerald-800 via-green-800 to-emerald-900 border-8 border-amber-900/60 shadow-2xl overflow-hidden"
          >
            {/* Table texture */}
            <div
              className="absolute inset-0 opacity-10"
              style={{
                backgroundImage:
                  "radial-gradient(circle at 30% 40%, rgba(255,255,255,0.4) 1px, transparent 1px)",
                backgroundSize: "12px 12px",
              }}
            />

            {/* Community Cards Area */}
            <div className="absolute top-1/3 left-1/2 -translate-x-1/2 -translate-y-1/2">
              <p className="text-white/40 text-xs font-mono text-center mb-2">Community Cards</p>
              <div className="flex gap-2">
                {communityCards.map((card, i) => (
                  <PlayingCard
                    key={i}
                    card={card}
                    faceUp={i < communityRevealed}
                    index={i}
                  />
                ))}
              </div>
            </div>

            {/* Player Hand Area */}
            <div className="absolute bottom-1/4 left-1/2 -translate-x-1/2">
              <p className="text-white/40 text-xs font-mono text-center mb-2">Your Hand</p>
              <div className="flex gap-2">
                {playerHand.map((card, i) => (
                  <PlayingCard key={i} card={card} faceUp={playerRevealed} index={i} />
                ))}
              </div>
            </div>

            {/* Pot area */}
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
              <div className="flex items-center gap-1 px-3 py-1 rounded-full bg-amber-500/20 border border-amber-500/30">
                <span className="text-amber-400 text-xs font-mono">POT: 24.5 SOL</span>
              </div>
            </div>
          </motion.div>

          {/* Controls */}
          <div className="mt-6 flex flex-wrap items-center justify-center gap-3">
            <Button
              variant="outline"
              onClick={revealCommunity}
              disabled={communityRevealed >= 5}
              className="border-neutral-700 text-white hover:bg-neutral-800"
            >
              <Eye className="size-4 mr-1" />
              Reveal Next ({communityRevealed}/5)
            </Button>

            {proofState === "pending" && (
              <Button
                onClick={generateProof}
                className="bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white"
              >
                <Zap className="size-4 mr-1" />
                Generate Proof
              </Button>
            )}

            {proofState === "verified" && (
              <Button
                variant="outline"
                onClick={revealHand}
                disabled={playerRevealed}
                className="border-emerald-600 text-emerald-400 hover:bg-emerald-950"
              >
                <EyeOff className="size-4 mr-1" />
                {playerRevealed ? "Hand Revealed" : "Reveal Hand"}
              </Button>
            )}
          </div>

          {/* Proof Animation */}
          <AnimatePresence>
            {proofState === "generating" && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: "auto" }}
                exit={{ opacity: 0, height: 0 }}
                className="flex flex-col items-center justify-center py-6 overflow-hidden"
              >
                <RotatingRings />
                <p className="mt-3 text-sm text-indigo-300 font-mono">
                  Generating zero-knowledge proof...
                </p>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Status Bar */}
          <div className="mt-4 flex flex-wrap items-center justify-between gap-3 p-3 rounded-lg border border-neutral-800 bg-neutral-900">
            <div className="flex items-center gap-2">
              <span className="text-xs text-white/40 font-mono">Proof State:</span>
              <Badge
                variant={
                  proofState === "verified"
                    ? "default"
                    : proofState === "generating"
                      ? "secondary"
                      : "outline"
                }
                className={
                  proofState === "verified"
                    ? "bg-emerald-600 text-white"
                    : proofState === "generating"
                      ? "bg-indigo-600 text-white"
                      : "border-neutral-600 text-white/60"
                }
              >
                {proofState === "pending" && "Pending"}
                {proofState === "generating" && "Generating"}
                {proofState === "verified" && "Proof Verified ✓"}
              </Badge>
            </div>
            <div className="flex items-center gap-2 text-xs text-white/30">
              <Shield className="size-3" />
              <span>Your hand is verified by zero-knowledge proof on Solana</span>
              <Info className="size-3" />
            </div>
          </div>

          {/* Reset */}
          <div className="mt-4 text-center">
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
        </CardContent>
      </Card>
    </div>
  );
}
