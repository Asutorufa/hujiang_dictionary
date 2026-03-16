from playwright.sync_api import sync_playwright
import time

def verify_feature(page):
    page.goto("http://localhost:4173")

    # Let's wait for body and check what's there
    page.wait_for_timeout(3000)
    page.screenshot(path="/home/jules/verification/debug_start.png", full_page=True)
    print("Saved debug_start.png")

    page.locator('text="Config"').first.click()
    page.wait_for_timeout(2000)

    if "login" in page.url:
        print("Logging in...")
        page.fill('input[type="password"]', 'mock_password')
        page.get_by_role("button", name="Login").click()
        page.wait_for_timeout(2000)

    page.locator('text="Config"').first.click()
    page.wait_for_timeout(2000)
    page.get_by_text("General").first.click()
    page.wait_for_timeout(2000)

    page.locator('text="Home"').first.click()
    page.wait_for_timeout(2000)

    # Click translate method dropdown
    page.locator('button[aria-haspopup="menu"]').first.click()
    page.wait_for_timeout(1000)

    # Select Gemini model
    page.get_by_role("menuitem", name="Gemini").click()
    page.wait_for_timeout(1000)

    # Click Show advanced settings
    page.locator('button:has-text("Show")').first.click()
    page.wait_for_timeout(1000)
    page.screenshot(path="/home/jules/verification/home_advanced.png", full_page=True)
    print("Saved home_advanced.png")

    # Select Google Search API
    page.locator('button:has-text("None")').click()
    page.wait_for_timeout(1000)
    page.screenshot(path="/home/jules/verification/home_dropdown_open.png", full_page=True)
    print("Saved home_dropdown_open.png")

    page.get_by_role("menuitem", name="Google Search API").click()
    page.wait_for_timeout(1000)
    page.screenshot(path="/home/jules/verification/home_search_selected.png", full_page=True)
    print("Saved home_search_selected.png")

if __name__ == "__main__":
    import subprocess
    import time
    # Try using build + preview instead of dev since dev had an empty screen issue
    server = subprocess.Popen(["npm", "run", "preview", "--", "--port", "4173", "--strictPort"], cwd="web")
    time.sleep(3) # wait for server

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(record_video_dir="/home/jules/verification/video", viewport={'width': 1280, 'height': 800})
        page = context.new_page()
        try:
            verify_feature(page)
        except Exception as e:
            print(f"Error: {e}")
            page.screenshot(path="/home/jules/verification/error_verify.png", full_page=True)
        finally:
            context.close()
            browser.close()
            server.terminate()
