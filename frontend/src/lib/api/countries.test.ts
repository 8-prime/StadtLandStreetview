import { expect, it } from "vitest";
import { countries, countryFlag, searchCountries } from "./countries";

it("lists unique ISO codes with flags and finds countries by name, word, or code", () => {
  expect(new Set(countries.map((c) => c.code)).size).toBe(countries.length);
  expect(countryFlag("DE")).toBe("🇩🇪");
  expect(searchCountries("germ").map((c) => c.code)).toEqual(["DE"]);
  expect(searchCountries("de").map((c) => c.code)).toContain("DE");
  expect(searchCountries("kingdom").map((c) => c.code)).toContain("GB");
  expect(searchCountries("germany", ["DE"])).toEqual([]);
  expect(searchCountries("  ")).toHaveLength(countries.length);
});
