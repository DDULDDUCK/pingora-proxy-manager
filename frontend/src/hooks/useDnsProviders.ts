import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";

export interface DnsProvider {
  id: number;
  name: string;
  provider_type: string;
  created_at: number;
}

export interface CreateDnsProviderReq {
  name: string;
  provider_type: string;
  credentials: string;
}

export function useDnsProviders() {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["dns-providers"],
    queryFn: async () => {
      return (await api.request("/dns-providers")) as DnsProvider[];
    },
  });

  const createMutation = useMutation({
    mutationFn: async (data: CreateDnsProviderReq) => {
      return api.request("/dns-providers", {
        method: "POST",
        body: JSON.stringify(data),
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["dns-providers"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: number) => {
      return api.request(`/dns-providers/${id}`, {
        method: "DELETE",
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["dns-providers"] });
    },
  });

  return {
    dnsProviders: query.data || [],
    isLoading: query.isLoading,
    isError: query.isError,
    createDnsProvider: createMutation.mutateAsync,
    deleteDnsProvider: deleteMutation.mutateAsync,
  };
}
