import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { toast } from "sonner";

export interface Location {
  path: string;
  target: string;
  scheme: "http" | "https";
  rewrite?: boolean;
}

export interface Host {
  domain: string;
  target: string;
  scheme: "http" | "https";
  ssl_forced?: boolean;
  redirect_to?: string | null;
  redirect_status?: number;
  locations?: Location[];
  access_list_id?: number | null;
}

export interface Stream {
  id: number;
  listen_port: number;
  forward_host: string;
  forward_port: number;
  protocol: "tcp" | "udp";
}

export function useHosts() {
  return useQuery<Host[]>({
    queryKey: ["hosts"],
    queryFn: () => api.request("/hosts"),
  });
}

export function useAddHost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (newHost: Partial<Host>) => 
      api.request("/hosts", { method: "POST", body: JSON.stringify(newHost) }),
    onSuccess: () => {
      toast.success("Host added");
      queryClient.invalidateQueries({ queryKey: ["hosts"] });
    },
    onError: () => toast.error("Failed to add host"),
  });
}

export function useDeleteHost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (domain: string) => 
      api.request(`/hosts/${domain}`, { method: "DELETE" }),
    onSuccess: () => {
      toast.success("Host deleted");
      queryClient.invalidateQueries({ queryKey: ["hosts"] });
    },
    onError: () => toast.error("Failed to delete host"),
  });
}

export function useAddLocation() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: ({ domain, location }: { domain: string, location: Location }) => 
            api.request(`/hosts/${domain}/locations`, { method: "POST", body: JSON.stringify(location) }),
        onSuccess: () => {
            toast.success("Location added");
            queryClient.invalidateQueries({ queryKey: ["hosts"] });
        },
        onError: () => toast.error("Failed to add location"),
    });
}

export function useDeleteLocation() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: ({ domain, path }: { domain: string, path: string }) => 
            api.request(`/hosts/${domain}/locations?path=${encodeURIComponent(path)}`, { method: "DELETE" }),
        onSuccess: () => {
            toast.success("Location deleted");
            queryClient.invalidateQueries({ queryKey: ["hosts"] });
        },
        onError: () => toast.error("Failed to delete location"),
    });
}

export function useIssueCert() {
    return useMutation({
        mutationFn: ({ domain, email }: { domain: string, email: string }) =>
            api.request("/certs", { method: "POST", body: JSON.stringify({ domain, email }) }),
        onSuccess: () => toast.success("Certificate request queued"),
        onError: () => toast.error("Failed to request certificate"),
    });
}

// --- Streams ---

export function useStreams() {
  return useQuery<Stream[]>({
    queryKey: ["streams"],
    queryFn: () => api.request("/streams"),
  });
}

export function useAddStream() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (newStream: Partial<Stream>) => 
      api.request("/streams", { method: "POST", body: JSON.stringify(newStream) }),
    onSuccess: () => {
      toast.success("Stream added");
      queryClient.invalidateQueries({ queryKey: ["streams"] });
    },
    onError: () => toast.error("Failed to add stream"),
  });
}

export function useDeleteStream() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (port: number) => 
      api.request(`/streams/${port}`, { method: "DELETE" }),
    onSuccess: () => {
      toast.success("Stream deleted");
      queryClient.invalidateQueries({ queryKey: ["streams"] });
    },
    onError: () => toast.error("Failed to delete stream"),
  });
}
