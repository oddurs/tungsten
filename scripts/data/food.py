#!/usr/bin/env python3
"""Generates data/food.toml from USDA FoodData Central (SR Legacy).

    python3 scripts/data/food.py                  # downloads the CSV bundle
    python3 scripts/data/food.py DIR              # reads an unzipped bundle

Each food is one USDA entry and one of its published household portions
("medium banana", "can of beer"). Mass is the portion's gram weight; energy,
caffeine and sugar are USDA's per-100 g values scaled to the portion and
rounded to three significant figures. Nothing else is assumed.
"""

import csv
import io
import sys
import urllib.request
import zipfile
from pathlib import Path

URL = "https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_sr_legacy_food_csv_2018-04.zip"
OUT = Path(__file__).resolve().parents[2] / "data" / "food.toml"

ENERGY, CAFFEINE, SUGARS = "1008", "1057", "2000"

# (display, plural, names, fdc_id, portion modifier, portion amount, volume,
#  default property, for-scale properties)
FOODS = [
    ("coffee", "cups of coffee", ["coffee", "coffees", "cup of coffee", "cups of coffee", "brewed coffee"],
     "171890", "cup (8 fl oz)", 1, "8 floz", "caffeine", ["volume"]),
    ("espresso", "shots of espresso", ["espresso", "espressos", "shot of espresso", "shots of espresso", "espresso shot"],
     "171891", "fl oz", 1, "1 floz", "caffeine", []),
    ("tea", "cups of tea", ["tea", "teas", "cup of tea", "cups of tea", "black tea"],
     "173227", "cup (8 fl oz)", 1, "8 floz", "caffeine", []),
    ("green tea", "cups of green tea", ["green tea", "cup of green tea", "cups of green tea"],
     "171917", "cup", 1, "1 cup", "caffeine", []),
    ("can of cola", "cans of cola", ["cola", "colas", "can of cola", "cans of cola"],
     "174852", "can or bottle (12 fl oz)", 1, "12 floz", "caffeine", ["volume"]),
    ("can of Red Bull", "cans of Red Bull", ["red bull", "red bulls", "can of red bull", "cans of red bull"],
     "173210", "can 8.4 fl oz", 1, "8.4 floz", "caffeine", []),
    ("can of beer", "cans of beer", ["beer", "beers", "can of beer", "cans of beer"],
     "168746", "can", 1, "12 floz", "volume", []),
    ("glass of wine", "glasses of wine", ["wine", "glass of wine", "glasses of wine"],
     "173190", "serving (5 fl oz)", 1, "5 floz", "volume", []),
    ("glass of milk", "glasses of milk", ["milk", "glass of milk", "glasses of milk", "cup of milk", "cups of milk"],
     "171265", "cup", 1, "1 cup", "volume", []),
    ("banana", "bananas", ["banana", "bananas"],
     "173944", 'medium (7" to 7-7/8" long)', 1, None, "mass", ["mass"]),
    ("apple", "apples", ["apple", "apples"],
     "171688", 'medium (3" dia)', 1, None, "mass", ["mass"]),
    ("orange", "oranges", ["orange", "oranges"],
     "169097", 'fruit (2-5/8" dia)', 1, None, "mass", []),
    ("egg", "eggs", ["egg", "eggs", "chicken egg", "chicken eggs", "large egg"],
     "171287", "large", 1, None, "mass", ["mass"]),
    ("strawberry", "strawberries", ["strawberry", "strawberries"],
     "167762", 'medium (1-1/4" dia)', 1, None, "mass", []),
    ("avocado", "avocados", ["avocado", "avocados"],
     "171706", "fruit, without skin and seed", 1, None, "mass", []),
    ("grape", "grapes", ["grape", "grapes"],
     "174683", "grapes", 10, None, "mass", []),
    ("blueberry", "blueberries", ["blueberry", "blueberries"],
     "171711", "berries", 50, None, "mass", []),
    ("cherry", "cherries", ["cherry", "cherries"],
     "171719", "cherry", 1, None, "mass", []),
    ("almond", "almonds", ["almond", "almonds"],
     "170567", "almond", 1, None, "mass", []),
    ("watermelon", "watermelons", ["watermelon", "watermelons"],
     "167765", 'melon (15" long x 7-1/2" dia)', 1, None, "mass", ["mass"]),
    ("pineapple", "pineapples", ["pineapple", "pineapples"],
     "169124", "fruit", 1, None, "mass", []),
    ("tomato", "tomatoes", ["tomato", "tomatoes"],
     "170457", 'medium whole (2-3/5" dia)', 1, None, "mass", []),
    ("lemon", "lemons", ["lemon", "lemons"],
     "167746", 'fruit (2-1/8" dia)', 1, None, "mass", []),
    ("peach", "peaches", ["peach", "peaches"],
     "169928", 'medium (2-2/3" dia)', 1, None, "mass", []),
    ("mango", "mangoes", ["mango", "mangoes", "mangos"],
     "169910", "fruit without refuse", 1, None, "mass", []),
    ("kiwi", "kiwis", ["kiwi", "kiwis", "kiwifruit"],
     "168153", 'fruit (2" dia)', 1, None, "mass", []),
    ("potato", "potatoes", ["potato", "potatoes"],
     "170026", 'Potato medium (2-1/4" to 3-1/4" dia)', 1, None, "mass", []),
    ("carrot", "carrots", ["carrot", "carrots"],
     "170393", "medium", 1, None, "mass", []),
    ("onion", "onions", ["onion", "onions"],
     "170000", 'medium (2-1/2" dia)', 1, None, "mass", []),
    ("cucumber", "cucumbers", ["cucumber", "cucumbers"],
     "168409", 'cucumber (8-1/4")', 1, None, "mass", []),
    ("head of lettuce", "heads of lettuce", ["lettuce", "head of lettuce", "heads of lettuce"],
     "169248", 'head, medium (6" dia)', 1, None, "mass", []),
    ("slice of bread", "slices of bread", ["slice of bread", "slices of bread", "bread"],
     "174924", "slice", 1, None, "mass", []),
    ("slice of cheddar", "slices of cheddar", ["slice of cheese", "slices of cheese", "slice of cheddar", "slices of cheddar"],
     "170899", "slice (1 oz)", 1, None, "mass", []),
    ("teaspoon of sugar", "teaspoons of sugar", ["teaspoon of sugar", "teaspoons of sugar"],
     "169655", "tsp", 1, None, "mass", []),
    ("sugar cube", "sugar cubes", ["sugar cube", "sugar cubes"],
     "169655", "serving 1 cube", 1, None, "mass", []),
    ("tablespoon of butter", "tablespoons of butter", ["tablespoon of butter", "tablespoons of butter"],
     "173410", "tbsp", 1, None, "mass", []),
    ("stick of butter", "sticks of butter", ["stick of butter", "sticks of butter"],
     "173410", "stick", 1, None, "mass", []),
    ("tablespoon of honey", "tablespoons of honey", ["tablespoon of honey", "tablespoons of honey"],
     "169640", "tbsp", 1, None, "mass", []),
    ("tablespoon of olive oil", "tablespoons of olive oil", ["tablespoon of olive oil", "tablespoons of olive oil"],
     "171413", "tablespoon", 1, None, "mass", []),
    ("cup of rice", "cups of rice", ["cup of rice", "cups of rice"],
     "169753", "cup", 1, None, "mass", []),
    ("doughnut", "doughnuts", ["doughnut", "doughnuts", "donut", "donuts"],
     "174990", 'doughnut medium (3-1/4" dia)', 1, None, "mass", []),
    ("chocolate bar", "chocolate bars", ["chocolate bar", "chocolate bars", "bar of chocolate", "bars of chocolate"],
     "167587", "bar (1.55 oz)", 1, None, "mass", []),
    ("Big Mac", "Big Macs", ["big mac", "big macs"],
     "170720", "item 7.6 oz", 1, None, "mass", []),
]


def sig3(x):
    """Three significant figures, without trailing noise."""
    if x == 0:
        return "0"
    s = f"{x:.3g}"
    if "e" in s:
        m, e = s.split("e")
        return f"{m}e{int(e)}"
    return s


def toml(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def load(src):
    if src:
        read = lambda name: (Path(src) / name).read_text()
    else:
        data = urllib.request.urlopen(URL).read()
        z = zipfile.ZipFile(io.BytesIO(data))
        root = z.namelist()[0].split("/")[0]
        read = lambda name: z.read(f"{root}/{name}").decode()
    foods = {r["fdc_id"]: r["description"] for r in csv.DictReader(io.StringIO(read("food.csv")))}
    ids = {f[3] for f in FOODS}
    nutrients = {}
    for r in csv.DictReader(io.StringIO(read("food_nutrient.csv"))):
        if r["fdc_id"] in ids and r["nutrient_id"] in (ENERGY, CAFFEINE, SUGARS):
            nutrients[(r["fdc_id"], r["nutrient_id"])] = float(r["amount"])
    portions = {}
    for r in csv.DictReader(io.StringIO(read("food_portion.csv"))):
        if r["fdc_id"] in ids:
            portions.setdefault(r["fdc_id"], []).append((float(r["amount"]), r["modifier"], float(r["gram_weight"])))
    return foods, nutrients, portions


def main():
    foods, nutrients, portions = load(sys.argv[1] if len(sys.argv) > 1 else None)
    out = [
        "# data/food.toml — generated by scripts/data/food.py. Do not edit;",
        "# change FOODS in the script and regenerate.",
        "#",
        "# Foods from USDA FoodData Central, SR Legacy (April 2018). Mass is a",
        "# published household portion; energy, caffeine and sugar are USDA's",
        "# per-100 g values scaled to that portion, to three significant figures.",
        "",
    ]
    for display, plural, names, fdc, modifier, amount, volume, default, scale in FOODS:
        match = [p for p in portions.get(fdc, []) if p[1] == modifier and p[0] == amount]
        if not match:
            sys.exit(f"{display}: FDC {fdc} has no portion {amount} × {modifier!r}")
        grams = match[0][2] / amount
        per100 = lambda n: nutrients.get((fdc, n))
        props = {"mass": f"{sig3(grams)} g"}
        if volume:
            props["volume"] = volume
        if per100(ENERGY) is not None:
            props["food energy"] = f"{sig3(per100(ENERGY) * grams / 100)} kcal"
        if per100(CAFFEINE):
            props["caffeine"] = f"{sig3(per100(CAFFEINE) * grams / 100)} mg"
        if per100(SUGARS):
            props["sugar"] = f"{sig3(per100(SUGARS) * grams / 100)} g"
        portion = f"{amount:g} × {modifier}" if amount != 1 else modifier
        source = (
            f"USDA FoodData Central, SR Legacy, FDC {fdc} '{foods[fdc]}': portion "
            f"'{portion}' = {match[0][2]:g} g; nutrients scaled from per-100 g values"
        )
        out.append("[[entity]]")
        out.append('kind = "item"')
        out.append(f"display = {toml(display)}")
        out.append(f"plural = {toml(plural)}")
        out.append(f"names = [{', '.join(toml(n) for n in names)}]")
        out.append(f"default = {toml(default)}")
        if scale:
            out.append(f"scale = [{', '.join(toml(s) for s in scale)}]")
        out.append('domain = "food"')
        out.append(f"source = {toml(source)}")
        out.append("[entity.props]")
        for k, v in props.items():
            out.append(f"{toml(k)} = {toml(v)}")
        out.append("")
    OUT.write_text("\n".join(out))
    print(f"wrote {OUT} ({len(FOODS)} foods)")


if __name__ == "__main__":
    main()
