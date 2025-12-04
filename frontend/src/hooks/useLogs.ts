import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export function useLogs() {
  return useQuery<string[]>({
    queryKey: ["logs"],
    queryFn: () => api.request("/logs?lines=200"),
    refetchInterval: 5000,
  });
}
