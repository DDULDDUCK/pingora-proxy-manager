import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface AuditLog {
  id: number;
  timestamp: number;
  username: string;
  action: string;
  resource_type: string;
  resource_id: string | null;
  details: string | null;
  ip_address: string | null;
}

export interface AuditLogQuery {
  limit?: number;
  offset?: number;
  username?: string;
  resource_type?: string;
}

export function useAuditLogs(query: AuditLogQuery = {}) {
  const params = new URLSearchParams();
  if (query.limit) params.append("limit", query.limit.toString());
  if (query.offset) params.append("offset", query.offset.toString());
  if (query.username) params.append("username", query.username);
  if (query.resource_type) params.append("resource_type", query.resource_type);

  const queryString = params.toString();
  const endpoint = queryString ? `/audit-logs?${queryString}` : "/audit-logs";

  return useQuery<AuditLog[]>({
    queryKey: ["auditLogs", query],
    queryFn: () => api.request(endpoint),
    refetchInterval: 30000, // 30초마다 새로고침
  });
}