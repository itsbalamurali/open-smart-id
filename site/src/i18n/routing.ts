import { defineRouting } from "next-intl/routing";

export const routing = defineRouting({
  locales: ["en", "et"],
  defaultLocale: "en",
  localePrefix: "as-needed",
});
