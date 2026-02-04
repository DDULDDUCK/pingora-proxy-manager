import { FileText } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { useLogs } from "@/hooks/useLogs";

export function LogsTab() {
  const { t } = useTranslation();
  const { data: logs } = useLogs();

  return (
    <Card className="bg-black text-white border-zinc-800">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-white">
           <FileText className="h-5 w-5" /> {t('logs.title')}
        </CardTitle>
        <CardDescription className="text-zinc-400">
           {t('logs.description')}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="h-[500px] overflow-y-auto font-mono text-xs space-y-1 p-2 bg-zinc-950 rounded border border-zinc-800">
          {!logs || logs.length === 0 ? (
             <div className="text-zinc-500 italic">{t('logs.noLogs')}</div>
          ) : (
             logs.map((line, i) => (
               <div key={i} className="break-all border-b border-zinc-900/50 pb-0.5 mb-0.5">
                 <span className="text-green-500 mr-2">$</span>
                 {line}
               </div>
             ))
          )}
        </div>
      </CardContent>
    </Card>
  );
}
