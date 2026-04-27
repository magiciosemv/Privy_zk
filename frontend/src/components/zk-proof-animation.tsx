"use client";

import { motion, AnimatePresence } from "framer-motion";
import { Check, X } from "lucide-react";

interface ZKProofAnimationProps {
  status: "idle" | "generating" | "verified" | "failed";
}

const statusLabels: Record<ZKProofAnimationProps["status"], string> = {
  idle: "Ready to generate proof",
  generating: "Generating proof...",
  verified: "Proof verified",
  failed: "Verification failed",
};

export function ZKProofAnimation({ status }: ZKProofAnimationProps) {
  return (
    <div className="flex flex-col items-center gap-3">
      <div className="relative flex items-center justify-center size-20">
        <AnimatePresence mode="wait">
          {status === "idle" && (
            <motion.div
              key="idle"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="size-16 rounded-full border-2 border-purple-500/40"
              style={{
                animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
                boxShadow: "0 0 20px rgba(168, 85, 247, 0.15)",
              }}
            />
          )}

          {status === "generating" && (
            <motion.div
              key="generating"
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              className="size-16"
            >
              <motion.div
                animate={{ rotate: 360 }}
                transition={{ repeat: Infinity, duration: 1.5, ease: "linear" }}
                className="size-full rounded-full border-2 border-transparent border-t-purple-400"
                style={{
                  boxShadow:
                    "0 0 30px rgba(168, 85, 247, 0.4), 0 0 60px rgba(139, 92, 246, 0.2)",
                }}
              />
            </motion.div>
          )}

          {status === "verified" && (
            <motion.div
              key="verified"
              initial={{ opacity: 0, scale: 0.5 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.5 }}
              transition={{ type: "spring", stiffness: 300, damping: 20 }}
              className="flex size-16 items-center justify-center rounded-full border-2 border-green-500/40 bg-green-500/10"
            >
              <motion.div
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{
                  type: "spring",
                  stiffness: 400,
                  damping: 15,
                  delay: 0.1,
                }}
              >
                <Check className="size-8 text-green-400" />
              </motion.div>
            </motion.div>
          )}

          {status === "failed" && (
            <motion.div
              key="failed"
              initial={{ opacity: 0, scale: 0.5 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.5 }}
              transition={{ type: "spring", stiffness: 300, damping: 20 }}
              className="flex size-16 items-center justify-center rounded-full border-2 border-red-500/40 bg-red-500/10"
            >
              <motion.div
                animate={{ x: [0, -4, 4, -4, 4, 0] }}
                transition={{ duration: 0.4, ease: "easeInOut" }}
              >
                <X className="size-8 text-red-400" />
              </motion.div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      <AnimatePresence mode="wait">
        <motion.p
          key={status}
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -4 }}
          className="text-sm text-muted-foreground"
        >
          {statusLabels[status]}
        </motion.p>
      </AnimatePresence>
    </div>
  );
}
