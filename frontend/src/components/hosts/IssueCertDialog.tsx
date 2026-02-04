import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useDnsProviders } from "@/hooks/useDnsProviders";
import { useCertificates } from "@/hooks/useCertificates";
import { toast } from "sonner";
import { RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";

interface IssueCertDialogProps {
  domain: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function IssueCertDialog({ domain, open, onOpenChange }: IssueCertDialogProps) {
  const { t } = useTranslation();
  const { dnsProviders } = useDnsProviders();
  const { requestCert } = useCertificates();

  const [email, setEmail] = useState("admin@example.com");
  const [providerId, setProviderId] = useState<string>("http");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleRequest = async () => {
    if (!domain) return;
    if (!email) {
      toast.error(t('certificates.emailRequired'));
      return;
    }

    setIsSubmitting(true);
    try {
      await requestCert({
        domain,
        email,
        provider_id: providerId === "http" ? undefined : Number(providerId),
      });
      toast.success(t('certificates.certRequested'));
      onOpenChange(false);
    } catch (e) {
      toast.error(t('certificates.certRequestFailed'));
      console.error(e);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>{t('certificates.issueCertificateTitle')}</DialogTitle>
          <DialogDescription>
            {t('certificates.requestLetsEncrypt')} <strong>{domain}</strong>.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="email">{t('certificates.emailAddress')}</Label>
            <Input
              id="email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder={t('certificates.emailPlaceholder')}
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="provider">{t('certificates.validationMethod')}</Label>
            <Select value={providerId} onValueChange={setProviderId}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="http">{t('certificates.useHttp01')}</SelectItem>
                {dnsProviders.map((p) => (
                  <SelectItem key={p.id} value={p.id.toString()}>
                    DNS-01 ({p.name} - {p.provider_type})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              {t('certificates.dns01Note')}
            </p>
          </div>
        </div>
        <DialogFooter>
          <Button onClick={handleRequest} disabled={isSubmitting}>
            {isSubmitting && (
              <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
            )}
            {t('certificates.requestCertificate')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
