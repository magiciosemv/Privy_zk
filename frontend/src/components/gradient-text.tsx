import { cn } from "@/lib/utils";

export function GradientText({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "bg-gradient-to-r from-purple-400 via-violet-500 to-indigo-400 bg-clip-text text-transparent",
        className
      )}
    >
      {children}
    </span>
  );
}
