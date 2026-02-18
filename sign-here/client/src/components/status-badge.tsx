/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */
import { Badge } from "@/components/ui/badge";

const statusConfig: Record<string, { label: string; className: string }> = {
  draft: { label: "Draft", className: "bg-muted text-muted-foreground" },
  sent: { label: "Sent", className: "bg-amber-600/10 text-amber-700 dark:text-amber-400" },
  pending: { label: "Pending", className: "bg-amber-500/10 text-amber-700 dark:text-amber-400" },
  signing: { label: "In Progress", className: "bg-amber-500/10 text-amber-700 dark:text-amber-400" },
  completed: { label: "Completed", className: "bg-emerald-600/10 text-emerald-700 dark:text-emerald-400" },
  declined: { label: "Declined", className: "bg-red-500/10 text-red-700 dark:text-red-400" },
  signed: { label: "Signed", className: "bg-emerald-600/10 text-emerald-700 dark:text-emerald-400" },
};

export function StatusBadge({ status }: { status: string }) {
  const config = statusConfig[status] || statusConfig.draft;

  return (
    <Badge
      variant="outline"
      className={`${config.className} border-transparent text-[10px] font-medium tracking-wide uppercase no-default-hover-elevate no-default-active-elevate`}
      data-testid={`badge-status-${status}`}
    >
      {config.label}
    </Badge>
  );
}
