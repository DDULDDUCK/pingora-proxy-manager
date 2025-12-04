import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface RealtimeStats {
  requests: number;
  bytes: number;
  status_2xx: number;
  status_4xx: number;
  status_5xx: number;
}

export interface HistoryStat {
  timestamp: number;
  total_requests: number;
  total_bytes: number;
  status_2xx: number;
  status_4xx: number;
  status_5xx: number;
}

export function useRealtimeStats() {
  return useQuery<RealtimeStats>({
    queryKey: ["stats", "realtime"],
    queryFn: () => api.request("/stats/realtime"),
    refetchInterval: 2000,
  });
}

export function useHistoryStats() {
  return useQuery<HistoryStat[]>({
    queryKey: ["stats", "history"],
    queryFn: async () => {
      const data = await api.request("/stats/history?hours=24");
      return data.map((d: any) => ({
        ...d,
        time: new Date(d.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
      }));
    },
    refetchInterval: 60000,
  });
}
