import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";

export interface Cert {
  id: number;
  domain: string;
  expires_at: number;
}

export interface CreateCertReq {
  domain: string;
  email: string;
  provider_id?: number;
}

export function useCertificates() {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["certs"],
    queryFn: async () => {
      return (await api.request("/certs")) as Cert[];
    },
  });

  const createMutation = useMutation({
    mutationFn: async (data: CreateCertReq) => {
      return api.request("/certs", {
        method: "POST",
        body: JSON.stringify(data),
      });
    },
    onSuccess: () => {
      // 인증서 발급은 비동기 백그라운드 작업이므로 즉시 반영되지 않을 수 있음.
      // 사용자에게 "요청됨" 알림을 보여주고, 나중에 목록을 갱신하도록 유도하거나 폴링을 고려해야 함.
      // 여기서는 일단 쿼리 무효화만 수행.
      queryClient.invalidateQueries({ queryKey: ["certs"] });
    },
  });

  return {
    certs: query.data || [],
    isLoading: query.isLoading,
    isError: query.isError,
    requestCert: createMutation.mutateAsync,
  };
}
