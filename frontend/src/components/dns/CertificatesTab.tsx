import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useDnsProviders, type CreateDnsProviderReq } from "../../hooks/useDnsProviders";
import { useCertificates, type CreateCertReq } from "../../hooks/useCertificates";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../ui/table";
import { Button } from "../ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
} from "../ui/dialog";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Textarea } from "../ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { Trash2, Plus, Shield, Globe, CheckCircle, AlertTriangle } from "lucide-react";
import { toast } from "sonner";

const PROVIDER_TEMPLATES: Record<string, string> = {
  cloudflare: "dns_cloudflare_api_token = YOUR_CLOUDFLARE_API_TOKEN",
  route53: `[default]
aws_access_key_id = YOUR_ACCESS_KEY_ID
aws_secret_access_key = YOUR_SECRET_ACCESS_KEY
# aws_session_token = YOUR_SESSION_TOKEN (Optional, for temporary credentials)`,
  google: `google_cloud_dns_service_account_json = { /* YOUR GCP SERVICE ACCOUNT JSON HERE */ }`, // Note: Certbot expects JSON content directly
  digitalocean: "dns_digitalocean_api_token = YOUR_DIGITALOCEAN_API_TOKEN",
};

export function CertificatesTab() {
  const { t } = useTranslation();
  
  return (
    <div className="p-4">
      <Tabs defaultValue="certs" className="space-y-4">
        <TabsList>
          <TabsTrigger value="certs">{t('certificates.issuedCertificates')}</TabsTrigger>
          <TabsTrigger value="providers">{t('certificates.dnsProviders')}</TabsTrigger>
        </TabsList>
        
        <TabsContent value="certs" className="space-y-4">
          <IssuedCertificates />
        </TabsContent>
        
        <TabsContent value="providers" className="space-y-4">
          <DnsProviders />
        </TabsContent>
      </Tabs>
    </div>
  );
}

function IssuedCertificates() {
  const { t } = useTranslation();
  const { certs, requestCert } = useCertificates();
  const { dnsProviders } = useDnsProviders();
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [newCert, setNewCert] = useState<CreateCertReq>({
    domain: "",
    email: "",
    provider_id: undefined,
  });

  const handleRequest = async () => {
    if (!newCert.domain || !newCert.email) {
      toast.error(t('certificates.domainEmailRequired'));
      return;
    }
    try {
      await requestCert(newCert);
      toast.success(t('certificates.certRequested'));
      setIsCreateOpen(false);
      setNewCert({ domain: "", email: "", provider_id: undefined });
    } catch (e) {
      toast.error(t('certificates.certRequestFailed'));
      console.error(e);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-lg font-semibold">{t('certificates.issuedCertificates')}</h2>
          <p className="text-sm text-muted-foreground">
            {t('certificates.issuedCertsDesc')}
          </p>
        </div>
        <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" /> {t('certificates.requestCertificate')}
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[500px]">
            <DialogHeader>
              <DialogTitle>{t('certificates.requestLetsEncrypt')}</DialogTitle>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="domain">{t('certificates.domainNames')}</Label>
                <Input
                  id="domain"
                  placeholder={t('certificates.domainPlaceholder')}
                  value={newCert.domain}
                  onChange={(e) => setNewCert({ ...newCert, domain: e.target.value })}
                />
                <p className="text-xs text-muted-foreground">
                  {t('certificates.wildcardNote')}
                </p>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="email">{t('certificates.emailAddress')}</Label>
                <Input
                  id="email"
                  type="email"
                  placeholder={t('certificates.emailPlaceholder')}
                  value={newCert.email}
                  onChange={(e) => setNewCert({ ...newCert, email: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="provider">{t('certificates.dnsProviderOptional')}</Label>
                <Select
                  value={newCert.provider_id?.toString()}
                  onValueChange={(val) =>
                    setNewCert({ ...newCert, provider_id: val === "http" ? undefined : Number(val) })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('certificates.useHttp01')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="http">{t('certificates.useHttp01NoDns')}</SelectItem>
                    {dnsProviders.map((p) => (
                      <SelectItem key={p.id} value={p.id.toString()}>
                        {p.name} ({p.provider_type})
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
              <Button onClick={handleRequest}>{t('certificates.request')}</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t('certificates.domain')}</TableHead>
              <TableHead>{t('certificates.expires')}</TableHead>
              <TableHead>{t('certificates.status')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {certs.length === 0 ? (
              <TableRow>
                <TableCell colSpan={3} className="text-center h-24 text-muted-foreground">
                  {t('certificates.noCertificatesFound')}
                </TableCell>
              </TableRow>
            ) : (
              certs.map((cert) => {
                // eslint-disable-next-line react-hooks/purity
                const now = Math.floor(Date.now() / 1000);
                const daysLeft = Math.floor((cert.expires_at - now) / 86400);
                const isExpired = daysLeft < 0;
                const isWarning = daysLeft < 30;

                return (
                  <TableRow key={cert.id}>
                    <TableCell className="font-medium flex items-center gap-2">
                      <Globe className="h-4 w-4 text-blue-500" />
                      {cert.domain}
                    </TableCell>
                    <TableCell>
                      {new Date(cert.expires_at * 1000).toLocaleDateString()} 
                      <span className="text-muted-foreground ml-2">
                        ({daysLeft} {t('certificates.daysLeft')})
                      </span>
                    </TableCell>
                    <TableCell>
                      {isExpired ? (
                        <span className="flex items-center text-red-500 gap-1">
                          <AlertTriangle className="h-4 w-4" /> {t('certificates.expired')}
                        </span>
                      ) : isWarning ? (
                        <span className="flex items-center text-yellow-500 gap-1">
                          <AlertTriangle className="h-4 w-4" /> {t('certificates.renewSoon')}
                        </span>
                      ) : (
                        <span className="flex items-center text-green-500 gap-1">
                          <CheckCircle className="h-4 w-4" /> {t('certificates.valid')}
                        </span>
                      )}
                    </TableCell>
                  </TableRow>
                );
              })
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}

function DnsProviders() {
  const { t } = useTranslation();
  const { dnsProviders, createDnsProvider, deleteDnsProvider } = useDnsProviders();
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [newProvider, setNewProvider] = useState<CreateDnsProviderReq>({
    name: "",
    provider_type: "cloudflare",
    credentials: PROVIDER_TEMPLATES["cloudflare"],
  });

  const handleCreate = async () => {
    if (!newProvider.name || !newProvider.credentials) {
      toast.error(t('certificates.fillAllFields'));
      return;
    }
    try {
      await createDnsProvider(newProvider);
      toast.success(t('certificates.dnsProviderCreated'));
      setIsCreateOpen(false);
      setNewProvider({
        name: "",
        provider_type: "cloudflare",
        credentials: PROVIDER_TEMPLATES["cloudflare"],
      });
    } catch (e) {
      toast.error(t('certificates.dnsProviderCreateFailed'));
      console.error(e);
    }
  };

  const handleDelete = async (id: number) => {
    if (confirm(t('certificates.deleteDnsProviderConfirm'))) {
      try {
        await deleteDnsProvider(id);
        toast.success(t('certificates.dnsProviderDeleted'));
      } catch (e) {
        toast.error(t('certificates.dnsProviderDeleteFailed'));
        console.error(e);
      }
    }
  };

  const handleTypeChange = (val: string) => {
    setNewProvider({
      ...newProvider,
      provider_type: val,
      credentials: PROVIDER_TEMPLATES[val] || "",
    });
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-lg font-semibold">{t('certificates.dnsProviders')}</h2>
          <p className="text-sm text-muted-foreground">
            {t('certificates.dnsProvidersDesc')}
          </p>
        </div>
        <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
          <DialogTrigger asChild>
            <Button variant="outline">
              <Plus className="mr-2 h-4 w-4" /> {t('certificates.addProvider')}
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[500px]">
            <DialogHeader>
              <DialogTitle>{t('certificates.addDnsProvider')}</DialogTitle>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="name">{t('certificates.name')}</Label>
                <Input
                  id="name"
                  placeholder={t('certificates.namePlaceholder')}
                  value={newProvider.name}
                  onChange={(e) =>
                    setNewProvider({ ...newProvider, name: e.target.value })
                  }
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="type">{t('certificates.providerType')}</Label>
                <Select
                  value={newProvider.provider_type}
                  onValueChange={handleTypeChange}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('certificates.selectProvider')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="cloudflare">{t('certificates.cloudflare')}</SelectItem>
                    <SelectItem value="route53">{t('certificates.route53')}</SelectItem>
                    <SelectItem value="digitalocean">{t('certificates.digitalocean')}</SelectItem>
                    <SelectItem value="google">{t('certificates.googleCloudDns')}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="credentials">{t('certificates.credentials')}</Label>
                <Textarea
                  id="credentials"
                  className="font-mono text-sm h-[150px]"
                  value={newProvider.credentials}
                  onChange={(e) =>
                    setNewProvider({ ...newProvider, credentials: e.target.value })
                  }
                />
                <p className="text-xs text-muted-foreground">
                  {t('certificates.credentialsNote')}
                </p>
              </div>
            </div>
            <DialogFooter>
              <Button onClick={handleCreate}>{t('certificates.save')}</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t('certificates.name')}</TableHead>
              <TableHead>{t('certificates.type')}</TableHead>
              <TableHead>{t('certificates.createdAt')}</TableHead>
              <TableHead className="w-[100px]">{t('certificates.actions')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {dnsProviders.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} className="text-center h-24 text-muted-foreground">
                  {t('certificates.noDnsProvidersFound')}
                </TableCell>
              </TableRow>
            ) : (
              dnsProviders.map((provider) => (
                <TableRow key={provider.id}>
                  <TableCell className="font-medium flex items-center gap-2">
                    <Shield className="h-4 w-4 text-blue-500" />
                    {provider.name}
                  </TableCell>
                  <TableCell className="capitalize">
                    {provider.provider_type}
                  </TableCell>
                  <TableCell>
                    {new Date(provider.created_at * 1000).toLocaleDateString()}
                  </TableCell>
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => handleDelete(provider.id)}
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
