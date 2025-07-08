# Task: Test Cache Removal and Preserve Restartability

* **Task ID:** remove-discovery-cache-test_60.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - The new parallel directory walking implementation may be sufficiently performant to render the existing file discovery cache redundant for typical runs.
  - Removing the cache would simplify the codebase, reduce maintenance overhead, and eliminate a potential source of bugs related to cache coherency in the overlapped pipeline.
  - The primary historical purpose of the cache was to allow for restarting failed or partial processing runs. This functionality is critical and must be preserved.

* **Acceptance Criteria:**
  1.  A performance benchmark comparing the end-to-end processing time with the current parallel traversal + cache versus a version with the cache removed.
  2.  A design proposal for a new mechanism to ensure restartability. This could be a simple log of processed files or another method that is less complex than the current discovery cache.
  3.  An implementation of the proposed restart mechanism, demonstrating that a failed or interrupted run can be resumed without reprocessing completed files.
  4.  The new implementation must integrate cleanly with the overlapped discovery and processing pipeline.

* **Deliverables:**
  - A new module or set of functions for managing restartability.
  - Modifications to `main.rs` to remove the existing caching logic and integrate the new restart mechanism.
  - Benchmark results demonstrating the performance impact (positive or negative) of the changes.
  - Integration tests that verify the restart functionality.

* **References:**
  - `src/discovery.rs`
  - `src/main.rs`
  - `completed_tasks/overlapped-discovery-processing_57.stevejs.md`
