import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { toast } from "sonner";

export interface AccessList {
  id: number;
  name: string;
  clients: AccessListClient[];
  ips: AccessListIp[];
}

export interface AccessListClient {
  username: string;
  // password_hash is not sent to frontend
}

export interface AccessListIp {
  ip: string;
  action: "allow" | "deny";
}

export function useAccessLists() {
  return useQuery<AccessList[]>({
    queryKey: ["accessLists"],
    queryFn: () => api.request("/access-lists"),
  });
}

export function useAddAccessList() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => 
      api.request("/access-lists", { method: "POST", body: JSON.stringify({ name }) }),
    onSuccess: () => {
      toast.success("Access List created");
      queryClient.invalidateQueries({ queryKey: ["accessLists"] });
    },
    onError: () => toast.error("Failed to create access list"),
  });
}

export function useDeleteAccessList() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => 
      api.request(`/access-lists/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      toast.success("Access List deleted");
      queryClient.invalidateQueries({ queryKey: ["accessLists"] });
    },
    onError: () => toast.error("Failed to delete access list"),
  });
}

export function useAddClient() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, client }: { id: number, client: {username: string, password: string} }) => 
      api.request(`/access-lists/${id}/clients`, { method: "POST", body: JSON.stringify(client) }),
    onSuccess: () => {
      toast.success("Client added");
      queryClient.invalidateQueries({ queryKey: ["accessLists"] });
    },
    onError: () => toast.error("Failed to add client"),
  });
}

export function useRemoveClient() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, username }: { id: number, username: string }) => 
      api.request(`/access-lists/${id}/clients/${username}`, { method: "DELETE" }),
    onSuccess: () => {
      toast.success("Client removed");
      queryClient.invalidateQueries({ queryKey: ["accessLists"] });
    },
    onError: () => toast.error("Failed to remove client"),
  });
}

export function useAddIp() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ipRule }: { id: number, ipRule: AccessListIp }) => 
      api.request(`/access-lists/${id}/ips`, { method: "POST", body: JSON.stringify(ipRule) }),
    onSuccess: () => {
      toast.success("IP rule added");
      queryClient.invalidateQueries({ queryKey: ["accessLists"] });
    },
    onError: () => toast.error("Failed to add IP rule"),
  });
}

export function useRemoveIp() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ip }: { id: number, ip: string }) => 
      api.request(`/access-lists/${id}/ips/${encodeURIComponent(ip)}`, { method: "DELETE" }),
    onSuccess: () => {
      toast.success("IP rule removed");
      queryClient.invalidateQueries({ queryKey: ["accessLists"] });
    },
    onError: () => toast.error("Failed to remove IP rule"),
  });
}
