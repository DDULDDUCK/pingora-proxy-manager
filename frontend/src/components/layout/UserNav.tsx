import { LogOut, User as UserIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { User } from "@/hooks/useUsers";

interface UserNavProps {
  user: User;
  onLogout: () => void;
}

export function UserNav({ user, onLogout }: UserNavProps) {
  const { t } = useTranslation();

  const roleColors: Record<string, string> = {
    admin: "bg-destructive/10 text-destructive border-destructive/20",
    operator: "bg-blue-100 text-blue-700 border-blue-200 dark:bg-blue-900/30 dark:text-blue-400 dark:border-blue-800",
    viewer: "bg-muted text-muted-foreground border-border",
  };

  return (
    <div className="flex items-center gap-3 pl-2 border-l border-border ml-2">
      <div className="flex flex-col items-end hidden md:flex">
        <span className="text-sm font-medium leading-none">{user.username}</span>
        <span className="text-xs text-muted-foreground mt-1">
            <Badge variant="outline" className={`py-0 h-4 text-[10px] ${roleColors[user.role] || roleColors.viewer}`}>
                {user.role.toUpperCase()}
            </Badge>
        </span>
      </div>
      
      <div className="h-8 w-8 rounded-full bg-muted flex items-center justify-center overflow-hidden border border-border">
          <UserIcon className="h-4 w-4 text-muted-foreground" />
      </div>

      <Button variant="ghost" size="icon" onClick={onLogout} title={t('app.logout')}>
        <LogOut className="h-4 w-4" />
        <span className="sr-only">{t('app.logout')}</span>
      </Button>
    </div>
  );
}
