import { useState } from "react";
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
aws_access_key_id=YOUR_ACCESS_KEY_ID
aws_secret_access_key=YOUR_SECRET_ACCESS_KEY`,
  google: `google_cloud_dns_key_file = /path/to/your/service-account.json`,
  digitalocean: "dns_digitalocean_token = YOUR_DIGITALOCEAN_TOKEN",
};

export function CertificatesTab() {
  return (
    <div className="p-4">
      <Tabs defaultValue="certs" className="space-y-4">
        <TabsList>
          <TabsTrigger value="certs">Issued Certificates</TabsTrigger>
          <TabsTrigger value="providers">DNS Providers</TabsTrigger>
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
      toast.error("Domain and Email are required");
      return;
    }
    try {
      await requestCert(newCert);
      toast.success("Certificate requested. It may take a few minutes to issue.");
      setIsCreateOpen(false);
      setNewCert({ domain: "", email: "", provider_id: undefined });
    } catch (e) {
      toast.error("Failed to request certificate");
      console.error(e);
    }
  };

  const now = Math.floor(Date.now() / 1000);

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-lg font-semibold">Issued Certificates</h2>
          <p className="text-sm text-muted-foreground">
            Manage Let's Encrypt certificates for your domains.
          </p>
        </div>
        <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" /> Request Certificate
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[500px]">
            <DialogHeader>
              <DialogTitle>Request Let's Encrypt Certificate</DialogTitle>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="domain">Domain Names</Label>
                <Input
                  id="domain"
                  placeholder="e.g. example.com or *.example.com"
                  value={newCert.domain}
                  onChange={(e) => setNewCert({ ...newCert, domain: e.target.value })}
                />
                <p className="text-xs text-muted-foreground">
                  For wildcards (*.example.com), you MUST select a DNS Provider.
                </p>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="email">Email Address</Label>
                <Input
                  id="email"
                  type="email"
                  placeholder="admin@example.com"
                  value={newCert.email}
                  onChange={(e) => setNewCert({ ...newCert, email: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="provider">DNS Provider (Optional)</Label>
                <Select
                  value={newCert.provider_id?.toString()}
                  onValueChange={(val) =>
                    setNewCert({ ...newCert, provider_id: val === "http" ? undefined : Number(val) })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Use HTTP-01 Challenge" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="http">Use HTTP-01 Challenge (No DNS API)</SelectItem>
                    {dnsProviders.map((p) => (
                      <SelectItem key={p.id} value={p.id.toString()}>
                        {p.name} ({p.provider_type})
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  Select a DNS provider to use DNS-01 challenge (required for wildcards). 
                  Default is HTTP-01.
                </p>
              </div>
            </div>
            <DialogFooter>
              <Button onClick={handleRequest}>Request</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Domain</TableHead>
              <TableHead>Expires</TableHead>
              <TableHead>Status</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {certs.length === 0 ? (
              <TableRow>
                <TableCell colSpan={3} className="text-center h-24 text-muted-foreground">
                  No certificates found.
                </TableCell>
              </TableRow>
            ) : (
              certs.map((cert) => {
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
                        ({daysLeft} days left)
                      </span>
                    </TableCell>
                    <TableCell>
                      {isExpired ? (
                        <span className="flex items-center text-red-500 gap-1">
                          <AlertTriangle className="h-4 w-4" /> Expired
                        </span>
                      ) : isWarning ? (
                        <span className="flex items-center text-yellow-500 gap-1">
                          <AlertTriangle className="h-4 w-4" /> Renew Soon
                        </span>
                      ) : (
                        <span className="flex items-center text-green-500 gap-1">
                          <CheckCircle className="h-4 w-4" /> Valid
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
  const { dnsProviders, createDnsProvider, deleteDnsProvider } = useDnsProviders();
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [newProvider, setNewProvider] = useState<CreateDnsProviderReq>({
    name: "",
    provider_type: "cloudflare",
    credentials: PROVIDER_TEMPLATES["cloudflare"],
  });

  const handleCreate = async () => {
    if (!newProvider.name || !newProvider.credentials) {
      toast.error("Please fill in all fields");
      return;
    }
    try {
      await createDnsProvider(newProvider);
      toast.success("DNS Provider created");
      setIsCreateOpen(false);
      setNewProvider({
        name: "",
        provider_type: "cloudflare",
        credentials: PROVIDER_TEMPLATES["cloudflare"],
      });
    } catch (e) {
      toast.error("Failed to create DNS Provider");
      console.error(e);
    }
  };

  const handleDelete = async (id: number) => {
    if (confirm("Are you sure you want to delete this DNS Provider?")) {
      try {
        await deleteDnsProvider(id);
        toast.success("DNS Provider deleted");
      } catch (e) {
        toast.error("Failed to delete DNS Provider");
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
          <h2 className="text-lg font-semibold">DNS Providers</h2>
          <p className="text-sm text-muted-foreground">
            Manage API credentials for DNS challenges (required for Wildcard Certs).
          </p>
        </div>
        <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
          <DialogTrigger asChild>
            <Button variant="outline">
              <Plus className="mr-2 h-4 w-4" /> Add Provider
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[500px]">
            <DialogHeader>
              <DialogTitle>Add DNS Provider</DialogTitle>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="name">Name</Label>
                <Input
                  id="name"
                  placeholder="e.g. My Cloudflare"
                  value={newProvider.name}
                  onChange={(e) =>
                    setNewProvider({ ...newProvider, name: e.target.value })
                  }
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="type">Provider Type</Label>
                <Select
                  value={newProvider.provider_type}
                  onValueChange={handleTypeChange}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select a provider" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="cloudflare">Cloudflare</SelectItem>
                    <SelectItem value="route53">AWS Route53</SelectItem>
                    <SelectItem value="digitalocean">DigitalOcean</SelectItem>
                    <SelectItem value="google">Google Cloud DNS</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="credentials">Credentials (INI Format)</Label>
                <Textarea
                  id="credentials"
                  className="font-mono text-sm h-[150px]"
                  value={newProvider.credentials}
                  onChange={(e) =>
                    setNewProvider({ ...newProvider, credentials: e.target.value })
                  }
                />
                <p className="text-xs text-muted-foreground">
                  Enter the credentials in the format required by the Certbot DNS
                  plugin.
                </p>
              </div>
            </div>
            <DialogFooter>
              <Button onClick={handleCreate}>Save</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Type</TableHead>
              <TableHead>Created At</TableHead>
              <TableHead className="w-[100px]">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {dnsProviders.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} className="text-center h-24 text-muted-foreground">
                  No DNS Providers found.
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
