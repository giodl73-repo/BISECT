# Enhancement: Public Data and Dashboard Distribution

**Status**: Planned
**Created**: January 14, 2026
**Priority**: Medium (after core research complete)

## Overview

Make the redistricting data and interactive dashboard publicly accessible for researchers, journalists, and citizens to explore algorithmic redistricting results without needing to rebuild the expensive pipeline.

## Problem Statement

**Current State:**
- Full pipeline requires ~40GB of source data
- Takes 2-4 hours to run on high-end hardware
- Requires Python environment with specific dependencies
- Not accessible to non-technical users
- Data and results not easily shareable

**Goals:**
1. Publish generated outputs for all census years (2000, 2010, 2020)
2. Host interactive dashboard for public exploration
3. Minimize cost (ideally free or <$10/month)
4. Provide citation/attribution mechanisms
5. Enable reproducibility without full rebuild

## Use Cases

**Researchers:**
- Download district assignments for analysis
- Compare with other redistricting approaches
- Cite data with DOI in papers

**Journalists:**
- Explore algorithmic districts for their state
- Compare to enacted maps
- Generate maps/visualizations for articles

**Citizens/Advocates:**
- Understand what "fair" districts could look like
- Compare compactness scores
- Engage in redistricting debates with data

**Students:**
- Learn about redistricting algorithms
- Use as dataset for GIS/data science projects
- Reproducible research examples

## Part 1: Data Distribution Options

### Option A: Zenodo (RECOMMENDED for Data)

**Service**: Academic data repository (CERN/EU funded)

**Pros:**
- ✅ Free, unlimited storage
- ✅ Gets permanent DOI (citable)
- ✅ Designed for research data
- ✅ Version control (upload new versions)
- ✅ Integrates with GitHub releases
- ✅ Long-term preservation guarantee
- ✅ 50GB per dataset (can split if needed)

**Cons:**
- ❌ Upload can be slow for large files
- ❌ Requires account creation

**Cost:** Free

**What to upload:**
```
us_2020_v1.zip (estimated 5-10GB compressed)
├── states/
│   └── {state}/
│       ├── data/
│       │   ├── final_assignments.pkl
│       │   ├── district_summary.csv
│       │   └── district_cities.csv
│       └── maps/ (selected maps only)
├── data/
│   ├── us_all_districts.csv
│   └── us_district_summary.csv
└── index.html (dashboard)

us_2010_v1.zip (similar structure)
us_2000_v1.zip (similar structure)
```

**Example Citation:**
```
Della-Libera, G. (2026). Algorithmic Congressional Redistricting
Using Edge-Weighted Recursive Bisection (2020 Census) [Data set].
Zenodo. https://doi.org/10.5281/zenodo.XXXXXXX
```

### Option B: GitHub Releases

**Service**: GitHub's release artifact storage

**Pros:**
- ✅ Free
- ✅ Integrated with repository
- ✅ Automatic downloads via API
- ✅ Version tagged with git commits

**Cons:**
- ❌ 2GB per file limit (would need to split)
- ❌ No DOI (less academic credibility)
- ❌ Not designed for large datasets

**Cost:** Free

**Recommendation:** Use for small artifacts (CSVs, selected maps), not full datasets

### Option C: Archive.org

**Service**: Internet Archive (non-profit digital library)

**Pros:**
- ✅ Free, unlimited storage
- ✅ Permanent preservation
- ✅ No account required to download
- ✅ Supports large files

**Cons:**
- ❌ No DOI
- ❌ Less discoverable for academic use
- ❌ Slower downloads

**Cost:** Free

### Option D: AWS S3 + Requester Pays

**Service:** Amazon Web Services object storage

**Pros:**
- ✅ Scalable storage
- ✅ Fast downloads (CloudFront CDN)
- ✅ Pay only for what's used
- ✅ Requester pays = users pay for their downloads

**Cons:**
- ❌ Not free (storage: $0.023/GB/month)
- ❌ Requires AWS account
- ❌ Users need AWS account for requester pays
- ❌ More complex setup

**Cost Estimate:**
- Storage: 20GB × $0.023 = $0.46/month
- Bandwidth: Variable (can enable requester pays)
- **Total: ~$5-10/month**

### Recommended Data Distribution Strategy

**Primary: Zenodo**
- Upload complete datasets with DOI
- One dataset per census year
- Include README with methodology

**Secondary: GitHub Releases**
- Upload key CSVs (us_all_districts.csv, etc.)
- Upload selected visualizations
- Link to Zenodo for full data

**Documentation:**
- Add DATA_AVAILABILITY.md to repository
- Include download instructions
- Provide data dictionary

## Part 2: Dashboard Hosting Options

### Option A: GitHub Pages (RECOMMENDED)

**Service:** GitHub's static site hosting

**Pros:**
- ✅ Completely free
- ✅ HTTPS enabled automatically
- ✅ Custom domain support (yourproject.com)
- ✅ Automatic deployment from git
- ✅ Works perfectly for our static HTML dashboard
- ✅ No backend needed (data baked into HTML)
- ✅ Fast (GitHub's CDN)

**Cons:**
- ❌ 1GB repository size limit (our dashboard is ~5MB, totally fine)
- ❌ 100GB bandwidth/month soft limit (should be fine for reasonable traffic)
- ❌ Static only (no server-side processing)

**Cost:** Free

**Setup:**
1. Create `gh-pages` branch or use `docs/` folder
2. Copy dashboard HTML to repository
3. Enable GitHub Pages in settings
4. Access at `https://username.github.io/apportionment/`

**Custom Domain (Optional):**
- Register domain: ~$10-15/year (namecheap, google domains)
- Point to GitHub Pages with CNAME
- Example: `redistricting.giodl.com`

### Option B: Netlify

**Service:** Static site hosting with CI/CD

**Pros:**
- ✅ Free tier: 100GB bandwidth/month
- ✅ Automatic HTTPS
- ✅ Custom domain support
- ✅ Preview deployments (for testing)
- ✅ Form handling (could add contact form)
- ✅ Faster than GitHub Pages

**Cons:**
- ❌ Requires separate account
- ❌ 300 build minutes/month (more than enough)

**Cost:** Free tier is sufficient

### Option C: Vercel

**Service:** Similar to Netlify, owned by Next.js creators

**Pros:**
- ✅ Free tier: 100GB bandwidth/month
- ✅ Very fast (edge network)
- ✅ Automatic HTTPS
- ✅ Great analytics

**Cons:**
- ❌ Requires separate account

**Cost:** Free tier is sufficient

### Option D: AWS S3 + CloudFront

**Service:** Amazon's static hosting with CDN

**Pros:**
- ✅ Scalable
- ✅ Very fast (global CDN)
- ✅ No bandwidth limits

**Cons:**
- ❌ Not free
- ❌ More complex setup
- ❌ Need to manage AWS resources

**Cost Estimate:**
- S3 storage: ~$0.10/month (5MB)
- CloudFront: ~$1/month (100GB bandwidth)
- **Total: ~$1-2/month**

### Recommended Dashboard Hosting Strategy

**Primary: GitHub Pages**
```
Repository: github.com/yourname/apportionment
URL: https://yourname.github.io/apportionment/
```

**Advantages for our use case:**
- Dashboard is already a single static HTML file
- Data is baked in (no backend needed)
- Version controlled with git
- Free with reasonable traffic limits
- Easy to update (git push)

**Setup Steps:**
1. Create `docs/` folder in repository
2. Copy `outputs/us_2020_v1/index.html` → `docs/index.html`
3. Enable GitHub Pages in repository settings
4. Done!

**Optional: Custom Domain**
- Register `redistricting.yourname.com` ($12/year)
- Add CNAME record pointing to GitHub Pages
- Add custom domain in GitHub settings

## Part 3: Implementation Plan

### Phase 1: Prepare Data for Distribution

**Scripts to Create:**
```
scripts/distribution/
├── prepare_public_data.py       # Creates public-friendly data packages
├── validate_public_data.py      # Validates before upload
└── generate_readme.py           # Auto-generates data documentation
```

**What `prepare_public_data.py` does:**
1. Copy essential CSVs (district assignments, summaries)
2. Copy key maps (all_districts.png, selected analysis maps)
3. Remove unnecessary intermediate files
4. Compress to .zip files
5. Generate checksums (SHA256)
6. Create README with:
   - Data description
   - File structure
   - Column definitions
   - Methodology summary
   - Citation information
   - License (suggest CC-BY-4.0)

**Output:**
```
distribution/
├── us_2020_v1_data.zip (5-10GB compressed)
├── us_2020_v1_data.zip.sha256
├── README_2020.md
└── DATA_DICTIONARY.csv
```

### Phase 2: Upload to Zenodo

**Steps:**
1. Create Zenodo account (link to ORCID if available)
2. Create new upload
3. Upload .zip file + README
4. Add metadata:
   - Title
   - Authors
   - Description
   - Keywords (redistricting, census, algorithm, compactness)
   - Subjects (Political Science, Geography, Computer Science)
   - License (CC-BY-4.0 recommended)
5. Publish (get DOI)
6. Repeat for 2010, 2000

**Time estimate:** 2-4 hours (mostly upload time)

### Phase 3: Deploy Dashboard to GitHub Pages

**Steps:**
1. Create `docs/` directory in repository
2. Add script to copy dashboard:
```python
# scripts/distribution/deploy_dashboard.py
import shutil
from pathlib import Path

def deploy_to_github_pages(year='2020', version='v1'):
    """Copy dashboard to docs/ for GitHub Pages."""
    source = Path(f'outputs/us_{year}_{version}/index.html')
    dest = Path('docs/index.html')

    dest.parent.mkdir(exist_ok=True)
    shutil.copy2(source, dest)

    print(f"Dashboard deployed to {dest}")
    print("Commit and push to publish to GitHub Pages")
```
3. Enable GitHub Pages in repository settings
4. Commit and push
5. Test at `https://username.github.io/apportionment/`

**Time estimate:** 30 minutes

### Phase 4: Documentation Updates

**Files to Create/Update:**

1. **DATA_AVAILABILITY.md** (root directory)
```markdown
# Data Availability

## Complete Datasets

All generated redistricting data is available on Zenodo:

- 2020 Census: [![DOI](badge)](https://doi.org/10.5281/zenodo.XXX)
- 2010 Census: [![DOI](badge)](https://doi.org/10.5281/zenodo.XXX)
- 2000 Census: [![DOI](badge)](https://doi.org/10.5281/zenodo.XXX)

## Interactive Dashboard

Explore results: https://username.github.io/apportionment/

## Citation

If you use this data, please cite:
...
```

2. **Update README.md**
   - Add "Explore Results" section with dashboard link
   - Add "Download Data" section with Zenodo links
   - Add citation information

3. **Add LICENSE** (recommend MIT for code, CC-BY-4.0 for data)

## Part 4: Cost Analysis

### Scenario 1: Minimal (GitHub-only)
- **Data:** GitHub Releases (split files)
- **Dashboard:** GitHub Pages
- **Domain:** None (use github.io)
- **Cost:** $0/month
- **Limitations:** 2GB file size limits, basic analytics

### Scenario 2: Recommended (Zenodo + GitHub Pages)
- **Data:** Zenodo (with DOI)
- **Dashboard:** GitHub Pages
- **Domain:** Optional custom domain
- **Cost:** $0-1/month (optional domain only)
- **Advantages:** DOI for citations, unlimited data storage

### Scenario 3: Premium (Custom Domain + Analytics)
- **Data:** Zenodo
- **Dashboard:** Netlify (for analytics)
- **Domain:** Custom domain
- **Analytics:** Google Analytics (free)
- **Cost:** $1/month (domain only)
- **Advantages:** Better branding, traffic analytics

### Scenario 4: Enterprise (Full AWS)
- **Data:** S3 + CloudFront
- **Dashboard:** S3 + CloudFront
- **Domain:** Route53
- **Cost:** $5-15/month
- **Advantages:** Unlimited scale, full control
- **Disadvantages:** Overkill for this use case

## Part 5: Legal/Licensing Considerations

### Data License
**Recommend: CC-BY-4.0 (Creative Commons Attribution)**
- Allows reuse with attribution
- Standard for academic data
- Compatible with Zenodo

### Code License
**Current: Check repository**
**Recommend: MIT License**
- Permissive
- Allows commercial use
- Standard for open-source tools

### Source Data Attribution
**Must include:**
- Census Bureau attribution (public domain)
- Election data sources
- Any third-party data used

### Disclaimer
Add to README:
```
This redistricting algorithm is for research and educational purposes.
It does not constitute legal or political advice. Actual redistricting
involves legal constraints beyond algorithmic compactness.
```

## Part 6: Promotion/Outreach

Once public:
- Tweet from personal account (link to dashboard)
- Post to Reddit: r/dataisbeautiful, r/MapPorn, r/politics
- Share on GIS/data science forums
- Email to redistricting advocacy groups (Common Cause, FairVote, etc.)
- Submit to data aggregators (data.world, Kaggle, etc.)
- Add to awesome lists (awesome-gis, awesome-datasets)

## Part 7: Monitoring/Maintenance

**What to track:**
- GitHub Pages analytics (if custom domain)
- Zenodo download counts
- GitHub stars/forks
- Issues/questions from users

**Maintenance:**
- Update dashboard when new census data available
- Respond to issues/questions
- Keep dependencies updated (if backend added later)

## Success Metrics

**6 months after launch:**
- 100+ Zenodo downloads
- 1,000+ dashboard visits
- 10+ GitHub stars
- 1+ academic citation

**12 months after launch:**
- 500+ Zenodo downloads
- 5,000+ dashboard visits
- 50+ GitHub stars
- 5+ academic citations
- Media mention (blog, news article)

## Future Enhancements

**Once established:**
1. Add API for programmatic access
2. Enable district comparisons (algorithmic vs enacted)
3. Add "what-if" simulator (adjust parameters)
4. State-by-state deep dives
5. Historical comparison (2000 vs 2010 vs 2020)
6. Integration with GIS tools (ArcGIS, QGIS plugins)

## Decision Matrix

| Criterion | GitHub Only | Zenodo + GitHub (Recommended) | AWS |
|-----------|-------------|------------------------------|-----|
| **Cost** | Free | Free | $5-15/mo |
| **Data Size** | Limited (2GB) | Unlimited | Unlimited |
| **DOI/Citable** | No | Yes | No |
| **Speed** | Good | Good | Excellent |
| **Setup Time** | 1 hour | 2 hours | 4 hours |
| **Maintenance** | Low | Low | Medium |
| **Scalability** | Limited | Good | Excellent |
| **Recommended** | Small projects | Research (✓) | High traffic |

## Recommendation

**Start with: Zenodo + GitHub Pages**

**Rationale:**
1. Completely free
2. DOI makes it citable (important for academic impact)
3. GitHub Pages perfect for static dashboard
4. Easy to set up and maintain
5. Can always upgrade to paid hosting later if needed

**Timeline:**
- Phase 1 (Data prep): 1 day
- Phase 2 (Zenodo upload): 1 day (mostly upload time)
- Phase 3 (GitHub Pages): 2 hours
- Phase 4 (Documentation): 4 hours
- **Total: 2-3 days of work**

## Next Steps

1. Finish core research/papers
2. Create `scripts/distribution/prepare_public_data.py`
3. Generate final datasets for all census years
4. Create Zenodo account
5. Upload datasets (get DOIs)
6. Deploy dashboard to GitHub Pages
7. Update documentation
8. Announce publicly

---

**Questions to answer before implementation:**
- [ ] What license for code? (recommend MIT)
- [ ] What license for data? (recommend CC-BY-4.0)
- [ ] Custom domain desired? (cost: $12/year)
- [ ] Include contact form on dashboard?
- [ ] Add Google Analytics? (free, but tracks users)
- [ ] Which maps to include in data downloads? (balance size vs completeness)
