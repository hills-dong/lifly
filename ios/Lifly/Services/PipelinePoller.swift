import Foundation

/// Polls a pipeline until it reaches a terminal state or times out.
/// Submitting a raw input kicks off async LLM processing; the produced
/// DataObject only appears once the pipeline completes.
enum PipelinePoller {
    static func waitForCompletion(
        pipelineId: String,
        maxAttempts: Int = 20,
        interval: TimeInterval = 1.5
    ) async {
        for _ in 0..<maxAttempts {
            if let status = try? await LiflyAPI.pipelineStatus(id: pipelineId) {
                if status.status == "completed" || status.status == "failed" {
                    return
                }
            }
            try? await Task.sleep(nanoseconds: UInt64(interval * 1_000_000_000))
        }
    }
}
