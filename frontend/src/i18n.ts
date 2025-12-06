import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

import en from './locales/en.json';
import it from './locales/it.json';

/**
 * i18n configuration for Pingora Proxy Manager
 * Supports English and Italian languages
 */
i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: {
        translation: en
      },
      it: {
        translation: it
      }
    },
    fallbackLng: 'en',
    supportedLngs: ['en', 'it'],
    
    // Language detection configuration
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
      lookupLocalStorage: 'i18nextLng',
    },
    
    // Disable debug in production
    debug: false,
    
    interpolation: {
      escapeValue: false // React already escapes values
    },
    
    // React specific options
    react: {
      useSuspense: false
    }
  });

export default i18n;
