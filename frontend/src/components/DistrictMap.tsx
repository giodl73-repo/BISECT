/**
 * District map component using Leaflet.
 */
import { useEffect, useRef } from 'react'
import L from 'leaflet'
import 'leaflet/dist/leaflet.css'
import type { DistrictCollection, ColorMetric } from '../types/district'

interface DistrictMapProps {
  districts: DistrictCollection
  colorMetric: ColorMetric
  onDistrictClick?: (districtId: number) => void
  selectedDistrict?: number
}

export function DistrictMap({
  districts,
  colorMetric,
  onDistrictClick,
  selectedDistrict,
}: DistrictMapProps) {
  const mapRef = useRef<L.Map | null>(null)
  const layerRef = useRef<L.GeoJSON | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  // Initialize map
  useEffect(() => {
    if (!containerRef.current || mapRef.current) return

    mapRef.current = L.map(containerRef.current, {
      center: [39.8283, -98.5795], // Center of USA
      zoom: 4,
      zoomControl: true,
    })

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '© OpenStreetMap contributors',
      maxZoom: 19,
    }).addTo(mapRef.current)

    return () => {
      if (mapRef.current) {
        mapRef.current.remove()
        mapRef.current = null
      }
    }
  }, [])

  // Update districts layer
  useEffect(() => {
    if (!mapRef.current || !districts) return

    // Remove existing layer
    if (layerRef.current) {
      mapRef.current.removeLayer(layerRef.current)
    }

    // Add new layer
    layerRef.current = L.geoJSON(districts as any, {
      style: (feature) => {
        const district = feature?.properties
        if (!district) return {}

        const color = getDistrictColor(district, colorMetric)
        const isSelected = selectedDistrict === district.district_id

        return {
          fillColor: color,
          fillOpacity: isSelected ? 0.8 : 0.6,
          color: isSelected ? '#000' : '#666',
          weight: isSelected ? 3 : 1,
        }
      },
      onEachFeature: (feature, layer) => {
        const district = feature.properties

        // Bind popup
        layer.bindPopup(
          `
          <div class="p-2">
            <h3 class="font-bold text-sm mb-1">District ${district.district_id}</h3>
            <p class="text-xs"><strong>State:</strong> ${district.state}</p>
            <p class="text-xs"><strong>Population:</strong> ${district.population.toLocaleString()}</p>
            <p class="text-xs"><strong>Compactness:</strong> ${district.compactness_polsby_popper.toFixed(3)}</p>
          </div>
        `,
          { className: 'district-popup' }
        )

        // Click handler
        layer.on('click', () => {
          if (onDistrictClick) {
            onDistrictClick(district.district_id)
          }
        })

        // Highlight on hover
        layer.on('mouseover', function () {
          this.setStyle({
            weight: 3,
            color: '#000',
          })
        })

        layer.on('mouseout', function () {
          if (selectedDistrict !== district.district_id) {
            this.setStyle({
              weight: 1,
              color: '#666',
            })
          }
        })
      },
    }).addTo(mapRef.current)

    // Fit bounds to show all districts
    const bounds = layerRef.current.getBounds()
    if (bounds.isValid()) {
      mapRef.current.fitBounds(bounds, { padding: [20, 20] })
    }
  }, [districts, colorMetric, selectedDistrict, onDistrictClick])

  return (
    <div
      ref={containerRef}
      className="w-full h-full min-h-[500px] rounded-lg overflow-hidden border border-gray-300"
    />
  )
}

// Helper function to get color based on metric
function getDistrictColor(
  district: any,
  metric: ColorMetric
): string {
  switch (metric) {
    case 'compactness': {
      const value = district.compactness_polsby_popper
      // Red (low) -> Yellow -> Green (high)
      if (value < 0.1) return '#d32f2f'
      if (value < 0.2) return '#f57c00'
      if (value < 0.3) return '#fbc02d'
      if (value < 0.4) return '#afb42b'
      return '#388e3c'
    }
    case 'population': {
      const value = district.population
      // Blue gradient based on population
      if (value < 600000) return '#e3f2fd'
      if (value < 700000) return '#90caf9'
      if (value < 800000) return '#42a5f5'
      return '#1976d2'
    }
    case 'partisan': {
      const value = district.partisan_lean || 0
      // Red (R) -> Purple -> Blue (D)
      if (value < -0.1) return '#d32f2f' // Strong R
      if (value < -0.05) return '#e57373' // Lean R
      if (value < 0.05) return '#9c27b0' // Swing
      if (value < 0.1) return '#64b5f6' // Lean D
      return '#1976d2' // Strong D
    }
    case 'demographic': {
      // Default gray for demographic (would need specific metric)
      return '#9e9e9e'
    }
    default:
      return '#9e9e9e'
  }
}
