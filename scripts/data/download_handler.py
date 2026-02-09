#!/usr/bin/env python3
"""
Download utilities with retry logic and rate limiting.

Provides reusable functions for downloading data with:
- Exponential backoff
- 429 rate limit handling
- Multi-source fallback
- Progress reporting
- Cleanup on failure

Created: 2026-01-18 (Enhancement #48)
"""

import requests
import time
import random
from pathlib import Path
from typing import Optional, List, Callable
import zipfile
import shutil


def download_with_retry(
    url: str,
    output_path: Path,
    max_retries: int = 5,
    base_delay: float = 1.0,
    timeout: int = 30,
    progress_callback: Optional[Callable[[str], None]] = None
) -> bool:
    """
    Download a file with exponential backoff and 429 handling.

    Args:
        url: URL to download from
        output_path: Path to save file
        max_retries: Maximum number of retry attempts (default: 5)
        base_delay: Initial delay in seconds, doubles each retry (default: 1.0)
        timeout: Request timeout in seconds (default: 30)
        progress_callback: Optional callback function(message: str) for progress updates

    Returns:
        True if download succeeded, False otherwise

    Example:
        success = download_with_retry(
            'https://example.com/data.zip',
            Path('output/data.zip'),
            progress_callback=lambda msg: print(f"[INFO] {msg}")
        )
    """
    def log(msg: str):
        """Log message via callback or print."""
        if progress_callback:
            progress_callback(msg)

    # Ensure output directory exists
    output_path.parent.mkdir(parents=True, exist_ok=True)

    for attempt in range(max_retries):
        try:
            response = requests.get(url, timeout=timeout, stream=True)

            # Success!
            if response.status_code == 200:
                # Download with streaming to handle large files
                total_size = int(response.headers.get('content-length', 0))

                with open(output_path, 'wb') as f:
                    if total_size > 0:
                        downloaded = 0
                        chunk_size = 8192
                        for chunk in response.iter_content(chunk_size=chunk_size):
                            if chunk:
                                f.write(chunk)
                                downloaded += len(chunk)
                                # Optional progress reporting for large files
                                if total_size > 1_000_000:  # > 1MB
                                    progress = int(100 * downloaded / total_size)
                                    if downloaded % (chunk_size * 100) == 0:
                                        log(f"Downloading: {progress}%")
                    else:
                        # No content-length header, download all at once
                        f.write(response.content)

                log(f"Downloaded: {output_path.name}")
                return True

            # Rate limited - use exponential backoff with jitter
            elif response.status_code == 429:
                if attempt < max_retries - 1:
                    # Calculate delay: base * (2^attempt) + random jitter
                    delay = base_delay * (2 ** attempt) + random.uniform(0, 1)
                    log(f"Rate limited. Retry {attempt+1}/{max_retries} after {delay:.1f}s")
                    time.sleep(delay)
                    continue
                else:
                    log(f"Failed after {max_retries} attempts (rate limited)")
                    return False

            # Other HTTP error
            else:
                log(f"HTTP {response.status_code}: {url}")
                response.raise_for_status()

        except requests.exceptions.Timeout:
            if attempt < max_retries - 1:
                delay = base_delay * (2 ** attempt)
                log(f"Timeout. Retry {attempt+1}/{max_retries} after {delay:.1f}s")
                time.sleep(delay)
                continue
            else:
                log(f"Failed after {max_retries} attempts (timeout)")
                # Cleanup partial download
                if output_path.exists():
                    output_path.unlink()
                return False

        except requests.exceptions.RequestException as e:
            if attempt < max_retries - 1:
                delay = base_delay * (2 ** attempt)
                log(f"Error: {e}. Retry {attempt+1}/{max_retries} after {delay:.1f}s")
                time.sleep(delay)
                continue
            else:
                log(f"Failed after {max_retries} attempts: {e}")
                # Cleanup partial download
                if output_path.exists():
                    output_path.unlink()
                return False

        except Exception as e:
            log(f"Unexpected error: {e}")
            # Cleanup partial download
            if output_path.exists():
                output_path.unlink()
            return False

    return False


def download_with_fallback(
    urls: List[str],
    output_path: Path,
    max_retries: int = 3,
    progress_callback: Optional[Callable[[str], None]] = None
) -> bool:
    """
    Download from multiple fallback URLs, trying each in order.

    Useful when data source has moved or has multiple mirrors.

    Args:
        urls: List of URLs to try (in order)
        output_path: Path to save file
        max_retries: Retries per URL (default: 3)
        progress_callback: Optional callback for progress updates

    Returns:
        True if any URL succeeded, False if all failed

    Example:
        success = download_with_fallback(
            urls=[
                'https://primary.com/data.zip',
                'https://mirror1.com/data.zip',
                'https://mirror2.com/data.zip'
            ],
            output_path=Path('output/data.zip')
        )
    """
    def log(msg: str):
        if progress_callback:
            progress_callback(msg)

    for i, url in enumerate(urls, 1):
        log(f"Trying URL {i}/{len(urls)}: {url}")

        success = download_with_retry(
            url=url,
            output_path=output_path,
            max_retries=max_retries,
            progress_callback=progress_callback
        )

        if success:
            return True

        log(f"URL {i} failed, trying next...")

    log(f"All {len(urls)} URLs failed")
    return False


def download_and_extract_zip(
    url: str,
    extract_dir: Path,
    max_retries: int = 5,
    keep_zip: bool = False,
    progress_callback: Optional[Callable[[str], None]] = None
) -> bool:
    """
    Download ZIP file and extract contents.

    Args:
        url: URL of ZIP file
        extract_dir: Directory to extract files into
        max_retries: Maximum retry attempts (default: 5)
        keep_zip: Keep ZIP file after extraction (default: False)
        progress_callback: Optional callback for progress updates

    Returns:
        True if download and extraction succeeded, False otherwise

    Example:
        success = download_and_extract_zip(
            'https://example.com/data.zip',
            extract_dir=Path('output/extracted/')
        )
    """
    def log(msg: str):
        if progress_callback:
            progress_callback(msg)

    # Create temp directory for ZIP
    temp_dir = extract_dir.parent / f".download_temp_{extract_dir.name}"
    temp_dir.mkdir(parents=True, exist_ok=True)

    zip_filename = Path(url).name
    zip_path = temp_dir / zip_filename

    try:
        # Download ZIP
        log(f"Downloading ZIP: {zip_filename}")
        success = download_with_retry(
            url=url,
            output_path=zip_path,
            max_retries=max_retries,
            progress_callback=progress_callback
        )

        if not success:
            log("ZIP download failed")
            return False

        # Extract ZIP
        log(f"Extracting ZIP to: {extract_dir}")
        extract_dir.mkdir(parents=True, exist_ok=True)

        with zipfile.ZipFile(zip_path, 'r') as zip_ref:
            zip_ref.extractall(extract_dir)

        log(f"Extracted {len(zip_ref.namelist())} files")

        # Optionally keep ZIP in final location
        if keep_zip:
            final_zip_path = extract_dir / zip_filename
            shutil.move(str(zip_path), str(final_zip_path))
            log(f"Saved ZIP: {final_zip_path}")

        return True

    except zipfile.BadZipFile as e:
        log(f"Bad ZIP file: {e}")
        return False

    except Exception as e:
        log(f"Extraction error: {e}")
        return False

    finally:
        # Cleanup temp directory
        if temp_dir.exists():
            shutil.rmtree(temp_dir, ignore_errors=True)


def cleanup_on_failure(file_path: Path):
    """
    Delete a file (cleanup partial download).

    Args:
        file_path: Path to file to delete
    """
    if file_path.exists():
        try:
            file_path.unlink()
        except Exception:
            # Ignore errors during cleanup
            pass


def check_existing_file(
    file_path: Path,
    force: bool = False,
    progress_callback: Optional[Callable[[str], None]] = None
) -> bool:
    """
    Check if file exists and should be skipped.

    Args:
        file_path: Path to check
        force: Force redownload even if exists (default: False)
        progress_callback: Optional callback for skip message

    Returns:
        True if should skip (file exists and force=False), False otherwise
    """
    if file_path.exists() and not force:
        if progress_callback:
            progress_callback(f"[SKIP] Already exists: {file_path.name}")
        return True
    return False


def adaptive_rate_limit(
    response_headers: dict,
    base_delay: float = 1.0
) -> float:
    """
    Calculate adaptive delay based on response headers.

    Some APIs provide rate limit info in headers (e.g., X-RateLimit-Remaining).
    This function can be extended to parse those headers.

    Args:
        response_headers: HTTP response headers dict
        base_delay: Base delay in seconds

    Returns:
        Recommended delay in seconds
    """
    # Check for common rate limit headers
    remaining = response_headers.get('X-RateLimit-Remaining')
    reset_time = response_headers.get('X-RateLimit-Reset')

    if remaining is not None:
        try:
            remaining_int = int(remaining)
            if remaining_int < 10:
                # Low remaining requests, slow down
                return base_delay * 2
            elif remaining_int < 50:
                return base_delay * 1.5
        except ValueError:
            pass

    return base_delay
