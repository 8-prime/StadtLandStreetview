// ISO 3166-1 alpha-2 codes (plus XK for Kosovo). Names come from the browser.
const codes =
  `AD AE AF AG AI AL AM AO AQ AR AS AT AU AW AX AZ BA BB BD BE BF BG BH BI BJ BL BM BN BO BQ
BR BS BT BV BW BY BZ CA CC CD CF CG CH CI CK CL CM CN CO CR CU CV CW CX CY CZ DE DJ DK DM DO DZ EC EE EG EH
ER ES ET FI FJ FK FM FO FR GA GB GD GE GF GG GH GI GL GM GN GP GQ GR GS GT GU GW GY HK HM HN HR HT HU ID IE IL
IM IN IO IQ IR IS IT JE JM JO JP KE KG KH KI KM KN KP KR KW KY KZ LA LB LC LI LK LR LS LT LU LV LY MA MC MD ME
MF MG MH MK ML MM MN MO MP MQ MR MS MT MU MV MW MX MY MZ NA NC NE NF NG NI NL NO NP NR NU NZ OM PA PE PF PG PH
PK PL PM PN PR PS PT PW PY QA RE RO RS RU RW SA SB SC SD SE SG SH SI SJ SK SL SM SN SO SR SS ST SV SX SY SZ TC
TD TF TG TH TJ TK TL TM TN TO TR TT TV TW TZ UA UG UM US UY UZ VA VC VE VG VI VN VU WF WS XK YE YT ZA ZM ZW`.split(
    /\s+/,
  );

let names: Intl.DisplayNames | undefined;
try {
  names = new Intl.DisplayNames(["en"], { type: "region", fallback: "code" });
} catch {
  /* Older browsers show codes. */
}

export interface Country {
  code: string;
  name: string;
  flag: string;
}

export const countryName = (code: string) => names?.of(code) ?? code;
export const countryFlag = (code: string) =>
  String.fromCodePoint(
    ...[...code].map((letter) => 0x1f1a5 + letter.charCodeAt(0)),
  );

export const countries: Country[] = codes
  .map((code) => ({ code, name: countryName(code), flag: countryFlag(code) }))
  .sort((a, b) => a.name.localeCompare(b.name, "en"));

export function searchCountries(
  query: string,
  exclude: readonly string[] = [],
) {
  const needle = query.trim().toLocaleLowerCase("en");
  return countries.filter(
    (country) =>
      !exclude.includes(country.code) &&
      (!needle ||
        country.code.toLowerCase() === needle ||
        country.name
          .toLocaleLowerCase("en")
          .split(/[\s,()-]+/)
          .some((word) => word.startsWith(needle)) ||
        country.name.toLocaleLowerCase("en").startsWith(needle)),
  );
}
