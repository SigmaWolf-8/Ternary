// Auto-generated — Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending. See LICENSE in the repository root for full terms.
import { cn } from "@/lib/utils"

function Skeleton({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("animate-pulse rounded-md bg-muted", className)}
      {...props}
    />
  )
}

export { Skeleton }
