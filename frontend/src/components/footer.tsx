import { Badge } from "@/components/ui/badge";

const techBadges = [
  "Solana",
  "Anchor",
  "Groth16",
  "Poseidon",
  "Next.js",
  "Framer Motion",
];

export function Footer() {
  return (
    <footer className="border-t border-white/10 py-8">
      <div className="mx-auto flex max-w-6xl flex-col items-center gap-4 px-4">
        <p className="text-center text-sm text-muted-foreground">
          Built on Solana · Powered by Zero-Knowledge Proofs · © 2026 Privy SVM
        </p>
        <div className="flex flex-wrap items-center justify-center gap-2">
          {techBadges.map((name) => (
            <Badge key={name} variant="outline">
              {name}
            </Badge>
          ))}
        </div>
      </div>
    </footer>
  );
}
