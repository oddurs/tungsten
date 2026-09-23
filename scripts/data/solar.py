#!/usr/bin/env python3
"""Generates data/solar.toml from NASA's NSSDCA planetary fact sheets.

    python3 scripts/data/solar.py            # fetches the sheets
    python3 scripts/data/solar.py DIR        # reads <body>.html files from DIR

The Sun, the eight planets, Pluto and 21 major moons. Every number is parsed
from a fact sheet; the only derived values are mean radii of irregular moons,
computed as the cube root of the three published semi-axes, and marked so.
"""

import html
import re
import sys
import urllib.request
from pathlib import Path

BASE = "https://nssdc.gsfc.nasa.gov/planetary/factsheet/"
OUT = Path(__file__).resolve().parents[2] / "data" / "solar.toml"
PAGES = ["sun", "mercury", "venus", "earth", "moon", "mars", "jupiter", "saturn",
         "uranus", "neptune", "pluto", "joviansat", "saturniansat", "uraniansat",
         "neptuniansat", "index"]

PLANETS = ["mercury", "venus", "earth", "mars", "jupiter", "saturn", "uranus", "neptune"]
MOONS = {
    "joviansat": ("Jupiter", ["Io", "Europa", "Ganymede", "Callisto"]),
    "saturniansat": ("Saturn", ["Mimas", "Enceladus", "Tethys", "Dione", "Rhea", "Titan", "Iapetus"]),
    "uraniansat": ("Uranus", ["Miranda", "Ariel", "Umbriel", "Titania", "Oberon"]),
    "neptuniansat": ("Neptune", ["Triton"]),
}

# Label on a single-body sheet -> (property, unit). "{e}" takes the power of
# ten printed in the label, e.g. "Mass (1024 kg)" is 10^24 kg.
FIELDS = [
    (r"Mass \(10(\d+) kg\)", "mass", "e{e} kg"),
    (r"Volumetric mean radius \(km\)", "radius", " km"),
    (r"Equatorial radius \(km\)", "equatorial radius", " km"),
    (r"Mean density \(kg/m3\)", "density", " kg/m^3"),
    # Rocky bodies: at the surface. Gas giants: at the 1 bar level.
    (r"(?i)(surface )?gravity.*\(m/s2\)", "gravity", " m/s^2"),
    (r"Escape velocity \(km/s\)", "escape velocity", " km/s"),
    (r"Semimajor axis \(106 km\)", "{distance}", "e6 km"),
    (r"Sidereal orbit period \(days\)", "orbital period", " d"),
    (r"Revolution period \(days\)", "orbital period", " d"),
    (r"Sidereal rotation period \(hrs\)", "rotation period", " h"),
    (r"Length of day \(hrs\)", "day length", " h"),
    (r"Mean orbital velocity \(km/s\)", "orbital velocity", " km/s"),
    (r"Obliquity to orbit \(deg\)", "axial tilt", " deg"),
    (r"Luminosity \(1024 J/s\)", "luminosity", "e24 W"),
    (r"Effective temperature \(K\)", "surface temperature", " K"),
    (r"Number of natural satellites", "moons", ""),
]


def text_of(page_html):
    t = re.sub(r"<[^>]+>", "", page_html)
    return html.unescape(t).replace("\xa0", " ")


def num(s):
    s = s.strip().replace(",", "").rstrip("*").rstrip("R").rstrip(".")
    return s if re.fullmatch(r"[-+]?\d+(\.\d+)?", s) else None


def pairs(text):
    """(label, first value) in reading order, for both sheet layouts."""
    lines = [l.strip() for l in text.split("\n")]
    out = []
    for i, l in enumerate(lines):
        m = re.match(r"^(.+?\)|Number of natural satellites)\s+([-+]?[\d,]+\.?\d*\**)\s*(.*)$", l)
        if m and num(m.group(2)):
            out.append((m.group(1).strip(), num(m.group(2))))
            continue
        if l and (l.endswith(")") or l == "Number of natural satellites"):
            for nxt in lines[i + 1 : i + 4]:
                if nxt:
                    if num(nxt):
                        out.append((l, num(nxt)))
                    break
    return out


def updated(text):
    m = re.search(r"Last Updated:\s*([^,\n]+)", text)
    return m.group(1).strip() if m else "undated"


def fields(text, distance):
    got = {}
    for label, value in pairs(text):
        for pattern, prop, unit in FIELDS:
            m = re.fullmatch(pattern, label)
            if m and prop.format(distance=distance) not in got:
                unit = unit.format(e=m.group(1) if m.groups() else "")
                got[prop.format(distance=distance)] = f"{value}{unit}".strip()
    return got


def overview_temperatures(text):
    """Mean temperature row of the overview table, keyed by body."""
    t = text.split("\n")
    names = ["mercury", "venus", "earth", "moon", "mars", "jupiter", "saturn", "uranus", "neptune", "pluto"]
    for i, l in enumerate(t):
        if l.strip().startswith("Mean Temperature (C)"):
            vals = [v.strip() for v in t[i + 1 : i + 60] if v.strip()][:10]
            return dict(zip(names, vals))
    return {}


def toml(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


# Bodies that lend their size to the "for scale" pod.
SCALE = {
    "the Sun": ["mass", "radius"],
    "Earth": ["mass", "radius"],
    "the Moon": ["mass", "radius"],
    "Jupiter": ["mass", "radius"],
}
PLURALS = {"the Sun": "Suns", "the Moon": "Moons", "Earth": "Earths"}


def entity(out, kind, display, names, source, props, facts=None, sources=None):
    out.append("[[entity]]")
    out.append(f"kind = {toml(kind)}")
    out.append(f"display = {toml(display)}")
    if display in PLURALS:
        out.append(f"plural = {toml(PLURALS[display])}")
    out.append(f"names = [{', '.join(toml(n) for n in names)}]")
    if display in SCALE:
        out.append(f"scale = [{', '.join(toml(p) for p in SCALE[display])}]")
        out.append('domain = "space"')
    out.append(f"source = {toml(source)}")
    out.append("[entity.props]")
    for k, v in props.items():
        out.append(f"{toml(k)} = {toml(v)}")
    if facts:
        out.append("[entity.facts]")
        for k, v in facts.items():
            out.append(f"{toml(k)} = {toml(v)}")
    if sources:
        out.append("[entity.sources]")
        for k, v in sources.items():
            out.append(f"{toml(k)} = {toml(v)}")
    out.append("")


def table_rows(page_html):
    rows = []
    for r in re.findall(r"<tr[^>]*>(.*?)</tr>", page_html, re.S):
        cells = [re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", "", c))).strip()
                 for c in re.findall(r"<t[dh][^>]*>(.*?)</t[dh]>", r, re.S)]
        if cells:
            rows.append(cells)
    return rows


def mean_radius(cell):
    parts = [float(p.replace(",", "")) for p in cell.split("x")]
    if len(parts) == 1:
        return cell.replace(",", "").rstrip("."), False
    r = (parts[0] * parts[1] * parts[2]) ** (1 / 3)
    return f"{r:.1f}", True


def main():
    src = Path(sys.argv[1]) if len(sys.argv) > 1 else None
    pages = {}
    for p in PAGES:
        name = "planets" if p == "index" else p
        if src:
            pages[p] = (src / f"{name}.html").read_text()
        else:
            url = BASE if p == "index" else f"{BASE}{p}fact.html"
            pages[p] = urllib.request.urlopen(url).read().decode("latin-1")
    temps = overview_temperatures(text_of(pages["index"]))
    overview = f"NASA NSSDCA Planetary Fact Sheet (nssdc.gsfc.nasa.gov/planetary/factsheet), updated {updated(text_of(pages['index']))}"

    def sheet(p):
        return f"NASA NSSDCA {p.capitalize()} Fact Sheet (nssdc.gsfc.nasa.gov/planetary/factsheet/{p}fact.html), updated {updated(text_of(pages[p]))}"

    out = [
        "# data/solar.toml — generated by scripts/data/solar.py. Do not edit;",
        "# change the script and regenerate.",
        "#",
        "# The Sun, planets, Pluto and major moons, from NASA's NSSDCA fact sheets.",
        "",
    ]

    sun_text = text_of(pages["sun"])
    sun = fields(sun_text, "distance from sun")
    sun.pop("distance from sun", None)
    t = re.search(r"Effective temperature:\s*([\d.]+)", sun_text)
    if t:
        sun["surface temperature"] = f"{t.group(1)} K"
    entity(out, "star", "the Sun", ["sun", "the sun", "sol"], sheet("sun"), sun)

    for p in PLANETS:
        props = fields(text_of(pages[p]), "distance from sun")
        facts = {"orbits": "the Sun"}
        sources = {}
        if p in temps and num(temps[p]):
            props["mean temperature"] = f"{num(temps[p])} °C"
            sources["mean temperature"] = overview
        names = [p, "the earth", "planet earth", "terra"] if p == "earth" else [p]
        if p == "earth":
            props["age"] = "4.54 Gyr"
            sources["age"] = "USGS, Age of the Earth (pubs.usgs.gov/gip/geotime/age.html): 4.54 Ga"
        if p in ("jupiter", "saturn", "uranus", "neptune") and "gravity" in props:
            sources["gravity"] = sheet(p) + "; at the 1 bar pressure level"
        entity(out, "planet", p.capitalize(), names, sheet(p), props, facts, sources)

    moon = fields(text_of(pages["moon"]), "distance from earth")
    moon_sources = {}
    if num(temps.get("moon", "")):
        moon["mean temperature"] = f"{num(temps['moon'])} °C"
        moon_sources["mean temperature"] = overview
    entity(out, "moon", "the Moon", ["moon", "the moon", "luna"], sheet("moon"), moon,
           {"orbits": "Earth"}, moon_sources)

    pluto_text = text_of(pages["pluto"])
    charon_at = pluto_text.find("Charon")
    pluto = fields(pluto_text[:charon_at], "distance from sun")
    pluto_sources = {}
    if num(temps.get("pluto", "")):
        pluto["mean temperature"] = f"{num(temps['pluto'])} °C"
        pluto_sources["mean temperature"] = overview
    entity(out, "dwarf planet", "Pluto", ["pluto"], sheet("pluto"), pluto,
           {"orbits": "the Sun"}, pluto_sources)

    # Charon: its own block on the Pluto sheet.
    ch = dict(pairs(pluto_text[charon_at:]))
    entity(out, "moon", "Charon", ["charon"], sheet("pluto"), {
        "mass": f"{num(ch['Mass (1021 kg)'])}e21 kg",
        "radius": f"{num(ch['Equatorial radius (km)'])} km",
        "density": f"{num(ch['Mean density (kg/m3)'])} kg/m^3",
        "gravity": f"{num(ch['Surface gravity (m/s2)'])} m/s^2",
        "escape velocity": f"{num(ch['Escape velocity (km/s)'])} km/s",
        "distance from pluto": f"{num(ch['Mean distance from Pluto (km)'])} km",
        "orbital period": f"{num(ch['Sidereal orbit period (days)'])} d",
    }, {"orbits": "Pluto"})

    # Phobos and Deimos: two columns on the Mars sheet.
    mars_text = text_of(pages["mars"])
    at = mars_text.find("Phobos")
    lines = [l.strip() for l in mars_text[at:].split("\n") if l.strip()]
    def cols(label):
        i = lines.index(label)
        return num(lines[i + 1]), num(lines[i + 2])
    axes = [cols(l) for l in ["Subplanetary axis radius (km)", "Along-orbit axis radius (km)", "Polar axis radius (km)"]]
    for k, name in enumerate(["Phobos", "Deimos"]):
        a, b, c = (float(x[k]) for x in axes)
        entity(out, "moon", name, [name.lower()], sheet("mars"), {
            "mass": f"{cols('Mass (1015 kg)')[k]}e15 kg",
            "radius": f"{(a * b * c) ** (1 / 3):.1f} km",
            "density": f"{cols('Mean density (kg/m3)')[k]} kg/m^3",
            "distance from mars": f"{cols('Semimajor axis* (km)')[k]} km",
            "orbital period": f"{cols('Sidereal orbit period (days)')[k]} d",
        }, {"orbits": "Mars"}, {"radius": sheet("mars") + "; mean of the three published semi-axes, (abc)^(1/3)"})

    for page, (parent, names) in MOONS.items():
        rows = table_rows(pages[page])
        bulk, orbit = {}, {}
        for r in rows:
            key = r[0].split(" (")[0]
            if key in names:
                (orbit if key in bulk else bulk)[key] = r
        for n in names:
            b, o = bulk[n], orbit[n]
            radius, derived = mean_radius(b[2])
            props = {"mass": f"{num(b[1])}e20 kg", "radius": f"{radius} km"}
            if num(b[3]):
                props["density"] = f"{num(b[3])} kg/m^3"
            props[f"distance from {parent.lower()}"] = f"{num(o[1])}e3 km"
            props["orbital period"] = f"{num(o[3])} d"
            src = f"NASA NSSDCA {parent}ian Satellite Fact Sheet (nssdc.gsfc.nasa.gov/planetary/factsheet/{page}fact.html), updated {updated(text_of(pages[page]))}".replace("Jupiterian", "Jovian").replace("Saturnian", "Saturnian").replace("Uranusian", "Uranian").replace("Neptuneian", "Neptunian")
            sources = {"radius": src + "; mean of the three published semi-axes, (abc)^(1/3)"} if derived else {}
            entity(out, "moon", n, [n.lower()], src, props, {"orbits": parent}, sources)

    OUT.write_text("\n".join(out))
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
