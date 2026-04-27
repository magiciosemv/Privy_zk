import Link from "next/link";
import { ArrowLeft } from "lucide-react";

export default function DemosLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-neutral-950 text-white">
      <div className="fixed top-0 left-0 z-50 p-4">
        <Link
          href="/"
          className="inline-flex items-center gap-2 text-sm text-white/60 hover:text-white transition-colors"
        >
          <ArrowLeft className="size-4" />
          Back to Home
        </Link>
      </div>
      {children}
    </div>
  );
}
