import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { enUS } from "./locales/en-us";
import { zhCN } from "./locales/zh-cn";

const LANGUAGE_STORAGE_KEY = "language";
const initialLanguage =
	localStorage.getItem(LANGUAGE_STORAGE_KEY) || navigator.language;

i18n.use(initReactI18next).init({
	resources: {
		"en-US": enUS,
		"zh-CN": zhCN,
	},
	lng: initialLanguage,
	fallbackLng: "en-US",
	initAsync: false,
	interpolation: {
		escapeValue: false,
	},
});

/**
 * Persists the active language whenever i18next reports a language change.
 *
 * @example
 * i18n.changeLanguage("zh-CN");
 */
i18n.on("languageChanged", (language) => {
	localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
});

export default i18n;
