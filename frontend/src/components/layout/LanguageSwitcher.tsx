import { useTranslation } from 'react-i18next';
import { Globe } from 'lucide-react';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from '@/components/ui/select';

/**
 * Supported language configuration
 */
interface Language {
  code: string;
  name: string;
  flag: string;
  nativeName: string;
}

const SUPPORTED_LANGUAGES: readonly Language[] = [
  { code: 'en', name: 'English', flag: '🇬🇧', nativeName: 'English' },
  { code: 'it', name: 'Italian', flag: '🇮🇹', nativeName: 'Italiano' },
  { code: 'ko', name: 'Korean', flag: '🇰🇷', nativeName: '한국어' },
] as const;

/**
 * Language switcher component for i18n
 * Allows users to change the application language
 */
export function LanguageSwitcher() {
  const { i18n, t } = useTranslation();

  const handleLanguageChange = (langCode: string) => {
    i18n.changeLanguage(langCode);
    localStorage.setItem('i18nextLng', langCode);
  };

  const currentLanguage = SUPPORTED_LANGUAGES.find(
    (lang) => lang.code === i18n.language
  ) || SUPPORTED_LANGUAGES[0];

  return (
    <Select value={i18n.language} onValueChange={handleLanguageChange}>
      <SelectTrigger className="w-auto px-2 gap-2 border-0 bg-transparent focus:ring-0 focus:ring-offset-0 hover:bg-accent/50 transition-colors" aria-label={t('app.selectLanguage')}>
        <Globe className="h-4 w-4" aria-hidden="true" />
        <span className="hidden sm:inline-block text-sm font-medium uppercase">{currentLanguage.code}</span>
      </SelectTrigger>

      <SelectContent align="end">
        {SUPPORTED_LANGUAGES.map((lang) => (
          <SelectItem key={lang.code} value={lang.code}>
            <span className="flex items-center gap-2">
              <span role="img" aria-label={lang.name}>{lang.flag}</span>
              <span>{lang.nativeName}</span>
            </span>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
