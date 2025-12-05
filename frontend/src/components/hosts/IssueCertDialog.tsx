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

interface IssueCertDialogProps {
  domain: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function IssueCertDialog({ domain, open, onOpenChange }: IssueCertDialogProps) {
  const { dnsProviders } = useDnsProviders();
  const { requestCert } = useCertificates();

  const [email, setEmail] = useState("admin@example.com");
  const [providerId, setProviderId] = useState<string>("http");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleRequest = async () => {
    if (!domain) return;
    if (!email) {
      toast.error("Email is required");
      return;
    }

    setIsSubmitting(true);
    try {
      await requestCert({
        domain,
        email,
        provider_id: providerId === "http" ? undefined : Number(providerId),
      });
      toast.success(`Certificate requested for ${domain}`);
      onOpenChange(false);
    } catch (e) {
      toast.error("Failed to request certificate");
      console.error(e);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Issue SSL Certificate</DialogTitle>
          <DialogDescription>
            Request a Let's Encrypt certificate for <strong>{domain}</strong>.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="email">Email Address</Label>
            <Input
              id="email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="admin@example.com"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="provider">Validation Method</Label>
            <Select value={providerId} onValueChange={setProviderId}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="http">HTTP-01 (Webroot)</SelectItem>
                {dnsProviders.map((p) => (
                  <SelectItem key={p.id} value={p.id.toString()}>
                    DNS-01 ({p.name} - {p.provider_type})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              Use DNS-01 for wildcard domains or if port 80 is blocked.
            </p>
          </div>
        </div>
        <DialogFooter>
          <Button onClick={handleRequest} disabled={isSubmitting}>
            {isSubmitting && (
              <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
            )}
            Request Certificate
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
