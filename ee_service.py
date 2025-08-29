import ee
import json
import time
import logging
from functools import lru_cache

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Initialize GEE
try:
    ee.Initialize(opt_url='https://earthengine-highvolume.googleapis.com')
    logger.info("GEE initialized")
except Exception as e:
    logger.error(f"Init failed: {str(e)}")
    raise

@lru_cache(maxsize=32)
def get_climate_data(year: int, model: str):
    start_time = time.time()
    
    try:
        # 1. Get image collection (optimized filter)
        coll = ee.ImageCollection('NASA/GDDP-CMIP6').filter(
            ee.Filter.And(
                ee.Filter.eq('model', model),
                ee.Filter.date(f'{year}-01-01', f'{year}-12-31')
            )
        ).select('tas')
        
        # 2. Compute mean temperature
        mean_temp = coll.mean().subtract(273.15)
        
        # 3. Get stats (with validation)
        stats = mean_temp.reduceRegion(
            reducer=ee.Reducer.mean(),
            geometry=ee.Geometry.Rectangle([-180, -90, 180, 90]),
            scale=50000,  # Balanced resolution
            maxPixels=1e10
        ).getInfo()
        
        # 4. Generate tiles
        map_id = mean_temp.getMapId({
            'min': -10,
            'max': 30,
            'palette': ['blue', 'red'],
            'dimensions': 1024
        })
        
        logger.info(f"Processed {year}-{model} in {time.time()-start_time:.2f}s")
        return {
            "tile_url": map_id['tile_fetcher'].url_format,
            "stats": stats,
            "processing_time": time.time() - start_time
        }
        
    except Exception as e:
        logger.error(f"Failed processing: {str(e)}")
        return {"error": str(e)}

if __name__ == '__main__':
    # Test with hardcoded values
    result = get_climate_data(2023, "ACCESS-CM2")
    print(json.dumps(result, indent=2))