# UI Screenshots

This directory should contain screenshots of the RustForge UI for documentation purposes.

## Required Screenshots

### 1. Home Dashboard (`home.png`)
- Shows the landing page with three action cards
- Highlights: Clean dark theme, cyan accents, clear call-to-action buttons
- Resolution: 1920x1080 or similar

### 2. Workflow Builder (`workflow-builder.png`)
- Shows the visual workflow canvas with 2-3 agent nodes
- Highlights: Drag-and-drop interface, connection lines, properties panel
- Should show: Toolbar, canvas with nodes, right-side properties panel
- Resolution: 1920x1080 or similar

### 3. Workflow Builder - Properties Panel (`workflow-properties.png`)
- Close-up of the properties panel when an agent node is selected
- Shows: Agent ID, Type, Task description fields
- Resolution: 800x600 or similar

### 4. Execution Monitor (`execution-monitor.png`)
- Shows real-time execution monitoring with event timeline
- Highlights: Status badge, control buttons, event stream
- Should show: Multiple events in timeline with different types
- Resolution: 1920x1080 or similar

### 5. Execution Monitor - Events (`execution-events.png`)
- Close-up of the event timeline showing various event types
- Shows: TaskStarted, TaskCompleted, TaskFailed events with details
- Resolution: 1200x800 or similar

### 6. Executions List (`executions-list.png`)
- Shows the executions history page with multiple executions
- Highlights: Status badges, timestamps, metadata
- Should show: Mix of completed, running, and failed executions
- Resolution: 1920x1080 or similar

### 7. Settings Page (`settings.png`)
- Shows the settings configuration interface
- Highlights: Form fields, sections, save button
- Resolution: 1920x1080 or similar

### 8. Toast Notifications (`toast-notifications.png`)
- Shows toast notifications in action (success, error, info)
- Can be composite image showing different toast types
- Resolution: 600x400 or similar

## How to Take Screenshots

### Manual Method (Recommended)

1. Start the backend:
   ```bash
   cargo run -- serve --port 3000
   ```

2. Open browser to `http://localhost:3000`

3. Navigate to each page and take screenshots:
   - Use browser's full-page screenshot feature (F12 → Ctrl+Shift+P → "Capture screenshot")
   - Or use OS screenshot tool (Windows: Win+Shift+S, Mac: Cmd+Shift+4)

4. Save screenshots with the filenames listed above

5. Optimize images:
   ```bash
   # Optional: compress images
   pngquant *.png --ext .png --force
   ```

### Automated Method (Advanced)

Use Playwright or Puppeteer to automate screenshot capture:

```javascript
// Example with Playwright
const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  
  await page.goto('http://localhost:3000');
  await page.screenshot({ path: 'home.png', fullPage: true });
  
  await page.goto('http://localhost:3000/workflows');
  await page.screenshot({ path: 'workflow-builder.png', fullPage: true });
  
  // ... more screenshots
  
  await browser.close();
})();
```

## Demo Video/GIF

A demo video or animated GIF should showcase the complete workflow:

### Recommended Tools

- **Screen Recording:**
  - Windows: Xbox Game Bar (Win+G), OBS Studio
  - Mac: QuickTime Player, ScreenFlow
  - Linux: SimpleScreenRecorder, OBS Studio

- **GIF Creation:**
  - [ScreenToGif](https://www.screentogif.com/) (Windows)
  - [Gifski](https://gif.ski/) (Mac)
  - [Peek](https://github.com/phw/peek) (Linux)
  - [ezgif.com](https://ezgif.com/) (Online converter)

### Demo Script (2 minutes)

1. **Home Page** (0:00-0:10)
   - Show landing page
   - Hover over action cards

2. **Create Workflow** (0:10-0:40)
   - Click "Create Workflow"
   - Add 2-3 agent nodes
   - Connect nodes with dependencies
   - Configure agent properties
   - Save workflow

3. **Execute Workflow** (0:40-0:50)
   - Navigate to Workflows list
   - Click "Execute" button
   - Show redirect to Execution Monitor

4. **Monitor Execution** (0:50-1:20)
   - Show real-time event updates
   - Highlight status changes
   - Show agent outputs in timeline
   - Demonstrate pause/resume (optional)

5. **View History** (1:20-1:35)
   - Navigate to Executions list
   - Show completed executions
   - Click to view detailed timeline

6. **Settings** (1:35-1:50)
   - Open Settings page
   - Show configuration options
   - Demonstrate save with toast notification

### GIF Optimization

Keep GIF file size reasonable:
- Resolution: 1280x720 or 1024x768
- Frame rate: 10-15 fps
- Duration: 30-60 seconds (shorter is better)
- Use tools like [gifsicle](https://www.lcdf.org/gifsicle/) to optimize:
  ```bash
  gifsicle -O3 --colors 256 input.gif -o output.gif
  ```

## Notes

- Use consistent browser window size for all screenshots
- Ensure UI is in a clean state (no console errors, proper data)
- Consider using dark mode for consistency
- Add annotations or highlights if needed for documentation
- Screenshots should be taken with actual workflow data, not empty states
