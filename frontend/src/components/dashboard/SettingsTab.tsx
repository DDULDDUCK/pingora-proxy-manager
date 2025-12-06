import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "sonner";
import { RefreshCw, Save } from "lucide-react";
import { api } from "@/lib/api";

export function SettingsTab() {
  const { t } = useTranslation();
  const [html, setHtml] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetchErrorPage();
  }, []);

  const fetchErrorPage = async () => {
    setLoading(true);
    try {
      const content = await api.request("/settings/error-page");
      setHtml(content || "");
    } catch {
      toast.error("Failed to load error page");
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    setLoading(true);
    try {
      await api.request("/settings/error-page", {
        method: "POST",
        body: JSON.stringify({ html }),
      });
      toast.success("Error page updated");
    } catch {
      toast.error("Failed to update error page");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="grid gap-6">
      <Card>
        <CardHeader>
          <CardTitle>{t('settings.title')}</CardTitle>
          <CardDescription>
            {t('settings.description')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4">
            <Textarea 
              className="font-mono min-h-[400px]" 
              value={html} 
              onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setHtml(e.target.value)} 
              placeholder="<html>...</html>"
            />
            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={fetchErrorPage} disabled={loading}>
                <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
                Reset
              </Button>
              <Button onClick={handleSave} disabled={loading}>
                <Save className="mr-2 h-4 w-4" />
                {t('settings.save')}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
