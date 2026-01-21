import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { toast } from "sonner";

export interface Location {
  path: string;
  target: string;
  scheme: "http" | "https";
  rewrite?: boolean;
  verify_ssl?: boolean;
  upstream_sni?: string;
}

export interface Header {
  id: number;
  name: string;
  value: string;
  target: "request" | "response";
}

export interface Host {
  domain: string;
  target: string;
  scheme: "http" | "https";
  ssl_forced?: boolean;
  verify_ssl?: boolean;
  upstream_sni?: string;
  redirect_to?: string | null;
  redirect_status?: number;
  locations?: Location[];
  access_list_id?: number | null;
  headers?: Header[]; // Added headers field
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

// New hooks for custom headers
export function useAddHostHeader() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ domain, header }: { domain: string, header: Omit<Header, 'id'> }) =>
      api.request(`/hosts/${domain}/headers`, { method: "POST", body: JSON.stringify(header) }),
    onSuccess: () => {
      toast.success("Header added");
      queryClient.invalidateQueries({ queryKey: ["hosts"] });
    },
    onError: () => toast.error("Failed to add header"),
  });
}

export function useDeleteHostHeader() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ domain, headerId }: { domain: string, headerId: number }) =>
      api.request(`/hosts/${domain}/headers/${headerId}`, { method: "DELETE" }),
    onSuccess: () => {
      toast.success("Header deleted");
      queryClient.invalidateQueries({ queryKey: ["hosts"] });
    },
    onError: () => toast.error("Failed to delete header"),
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
