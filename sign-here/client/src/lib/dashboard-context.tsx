import { createContext, useContext, useState, type ReactNode } from "react";

export type SortField = "title" | "date" | "status" | "updated";
export type SortDirection = "asc" | "desc";

interface DashboardFilters {
  searchQuery: string;
  setSearchQuery: (q: string) => void;
  statusFilter: string;
  setStatusFilter: (s: string) => void;
  wbsFilter: string;
  setWbsFilter: (w: string) => void;
  sortField: SortField;
  setSortField: (f: SortField) => void;
  sortDirection: SortDirection;
  setSortDirection: (d: SortDirection) => void;
  clearAll: () => void;
}

const DashboardContext = createContext<DashboardFilters | null>(null);

export function DashboardFilterProvider({ children }: { children: ReactNode }) {
  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");
  const [wbsFilter, setWbsFilter] = useState("all");
  const [sortField, setSortField] = useState<SortField>("date");
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc");

  const clearAll = () => {
    setSearchQuery("");
    setStatusFilter("all");
    setWbsFilter("all");
    setSortField("date");
    setSortDirection("desc");
  };

  return (
    <DashboardContext.Provider
      value={{
        searchQuery, setSearchQuery,
        statusFilter, setStatusFilter,
        wbsFilter, setWbsFilter,
        sortField, setSortField,
        sortDirection, setSortDirection,
        clearAll,
      }}
    >
      {children}
    </DashboardContext.Provider>
  );
}

export function useDashboardFilters() {
  const ctx = useContext(DashboardContext);
  if (!ctx) throw new Error("useDashboardFilters must be used within DashboardFilterProvider");
  return ctx;
}
