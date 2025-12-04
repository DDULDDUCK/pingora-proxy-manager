import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface User {
  id: number;
  username: string;
  role: string;
  created_at: number;
  last_login: number | null;
}

export interface CreateUserData {
  username: string;
  password: string;
  role?: string;
}

export interface UpdateUserData {
  password?: string;
  role?: string;
}

export interface ChangePasswordData {
  current_password: string;
  new_password: string;
}

export function useUsers() {
  return useQuery<User[]>({
    queryKey: ["users"],
    queryFn: () => api.request("/users"),
  });
}

export function useCurrentUser() {
  return useQuery<User>({
    queryKey: ["currentUser"],
    queryFn: () => api.request("/users/me"),
  });
}

export function useCreateUser() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: CreateUserData) =>
      api.request("/users", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["users"] });
    },
  });
}

export function useUpdateUser() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: UpdateUserData }) =>
      api.request(`/users/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["users"] });
    },
  });
}

export function useDeleteUser() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: number) =>
      api.request(`/users/${id}`, {
        method: "DELETE",
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["users"] });
    },
  });
}

export function useChangeOwnPassword() {
  return useMutation({
    mutationFn: (data: ChangePasswordData) =>
      api.request("/users/me/password", {
        method: "PUT",
        body: JSON.stringify(data),
      }),
  });
}