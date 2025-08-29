import ee
import json
from datetime import datetime
import argparse
import sys
import calendar

# -----------------------
# Parse CLI arguments
# -----------------------
parser = argparse.ArgumentParser(description="Generate yearly average LST and anomaly tiles via GEE")
parser.add_argument("--year", type=int, required=True, help="Target year for analysis")
args = parser.parse_args()

target_year = args.year
current_year = datetime.now().year
current_month = datetime.now().month

# Determine months to average
if target_year == current_year:
    months = list(range(1, current_month + 1))  # Year-to-date
else:
    months = list(range(1, 13))  # Full year

# -----------------------
# Authenticate Earth Engine
# -----------------------
service_account = 'climate@urban-heat-464110.iam.gserviceaccount.com'
key_file = 'C:/Users/amina/app/climate-backend/service_account.json'

try:
    credentials = ee.ServiceAccountCredentials(service_account, key_file)
    ee.Initialize(credentials)
except Exception as e:
    print(f"Failed to initialize Earth Engine: {e}", file=sys.stderr)
    sys.exit(1)

# -----------------------
# Configuration
# -----------------------
climatology_start = '2000-01-01'
climatology_end = '2025-07-31'

# -----------------------
# Utility Functions
# -----------------------
def resample_era5_to_modis(era5_image, modis_reference):
    return era5_image.resample('bilinear').reproject(
        crs=modis_reference.projection(),
        scale=1000
    )

def fill_modis_gaps_with_era5(modis_image, era5_image):
    """Fill MODIS gaps with ERA5 where MODIS has no data"""
    return modis_image.unmask(era5_image)

def get_monthly_composite(collection, year, month):
    days_in_month = calendar.monthrange(year, month)[1]
    start_date = f"{year}-{month:02d}-01"
    end_date = f"{year}-{month:02d}-{days_in_month}"
    monthly_collection = collection.filterDate(start_date, end_date)
    count = monthly_collection.size().getInfo()
    if count == 0:
        print(f"No data found for {year}-{month:02d}")
        return None
    return monthly_collection.mean()

def get_yearly_composite(collection, year, months):
    composites = []
    for m in months:
        monthly = get_monthly_composite(collection, year, m)
        if monthly:
            composites.append(monthly)
    if not composites:
        return None
    return ee.ImageCollection(composites).mean()

def get_tile_urls(image, vis_params):
    """Return both a URL template and a test URL with fixed z/x/y"""
    try:
        map_dict = image.getMapId(vis_params)
        url_template = map_dict['tile_fetcher'].url_format
        # Example test tile at zoom=2, x=1, y=1
        test_url = url_template.format(z=2, x=1, y=1)
        return url_template, test_url
    except Exception as e:
        print(f"Failed to get tile URL: {e}", file=sys.stderr)
        return None, None

# -----------------------
# Process MODIS Data
# -----------------------
print("Processing MODIS data...")
modis_collection = ee.ImageCollection('MODIS/061/MOD11A1').select('LST_Day_1km')
target_modis = get_yearly_composite(modis_collection, target_year, months)
if target_modis is None:
    print("No MODIS data available for the target year.", file=sys.stderr)
    sys.exit(1)
target_modis_lst = target_modis.multiply(0.02).subtract(273.15).rename('LST')

# MODIS climatology for anomaly
modis_climatology = modis_collection.filterDate(climatology_start, climatology_end)
modis_long_term_mean = modis_climatology.mean().multiply(0.02).subtract(273.15)
modis_long_term_std = modis_climatology.reduce(ee.Reducer.stdDev()).multiply(0.02)

# -----------------------
# Process ERA5 Data
# -----------------------
print("Processing ERA5 data...")
era5_collection = ee.ImageCollection('ECMWF/ERA5_LAND/DAILY_AGGR').select('temperature_2m')
target_era5 = get_yearly_composite(era5_collection, target_year, months)
if target_era5 is None:
    print("No ERA5 data available for the target year.", file=sys.stderr)
    sys.exit(1)
target_era5_temp = target_era5.subtract(273.15).rename('ERA5_LST')

# ERA5 climatology for anomaly
era5_climatology = era5_collection.filterDate(climatology_start, climatology_end)
era5_long_term_mean = era5_climatology.mean().subtract(273.15)
era5_long_term_std = era5_climatology.reduce(ee.Reducer.stdDev())

# -----------------------
# Data Fusion: ERA5 base + MODIS top
# -----------------------
fused_target_lst = fill_modis_gaps_with_era5(target_modis_lst, target_era5_temp)

# -----------------------
# Calculate anomalies
# -----------------------
global_anomaly = fused_target_lst.subtract(modis_long_term_mean).divide(modis_long_term_std).rename('T_Anomaly')
absolute_anomaly = fused_target_lst.subtract(modis_long_term_mean).rename('T_Anomaly_C')

# -----------------------
# Visualization parameters
# -----------------------
lst_vis = {"min": -20, "max": 50, "palette": ["040274","040281","0502a3","0502b8","0502ce","0502e6","0602ff","235cb1","307ef3","269db1","30c8e2","32d3ef","3be285","3ff38f","86e26f","3ae237","b5e22e","d6e21f","fff705","ffd611","ffb613","ff8b13","ff6e08","ff500d","ff0000","de0101","c21301","a71001","911003"]}
anomaly_vis = {"min": -3, "max": 3, "palette": ['0000ff','ffffff','ff0000']}
absolute_anomaly_vis = {"min": -5, "max": 5, "palette": ['0000ff','ffffff','ff0000']}

# -----------------------
# Generate tile URLs
# -----------------------
print("Generating tile URLs...")
lst_template, lst_test = get_tile_urls(fused_target_lst, lst_vis)
anomaly_template, anomaly_test = get_tile_urls(global_anomaly, anomaly_vis)
absolute_anomaly_template, absolute_anomaly_test = get_tile_urls(absolute_anomaly, absolute_anomaly_vis)

if None in [lst_template, anomaly_template, absolute_anomaly_template]:
    print("Tile generation failed", file=sys.stderr)
    sys.exit(1)

# -----------------------
# Save configuration
# -----------------------
config_data = {
"lst_tile_url": lst_template,
    "anomaly_tile_url": anomaly_template,
    "absolute_anomaly_tile_url": absolute_anomaly_template
}

config_path = f"C:/Users/amina/app/climate-backend/tiles/tile_config_{target_year}.json"
with open(config_path, "w") as f:
    json.dump(config_data, f, indent=2)

print("="*60)
print(f"TILE GENERATION COMPLETE for {target_year}")
print(f"Config file saved to: {config_path}")
print(f"LST Tile Template: {lst_template}")
print(f"Anomaly Tile Template: {anomaly_template}")
print(f"Absolute Anomaly Tile Template: {absolute_anomaly_template}")
print(f"Absolute Anomaly Test Tile: {absolute_anomaly_test}")
