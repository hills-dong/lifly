import { useState, useRef, useEffect } from 'react';
import { useWebSocket } from './useWebSocket';

export interface UsePipelineStatusResult {
  pipelineId: string;
  pipelineStatus: string;
  setPipelineId: (id: string) => void;
}

/**
 * Encapsulates the WebSocket listener that tracks a single pipeline execution.
 *
 * @param onCompleted  Optional callback invoked when the pipeline reaches "completed".
 */
export function usePipelineStatus(
  onCompleted?: () => void,
): UsePipelineStatusResult {
  const [pipelineId, setPipelineIdRaw] = useState('');
  const [pipelineStatus, setPipelineStatus] = useState('');

  // When a pipeline ID is set, automatically mark status as "submitted" to
  // give the user immediate feedback before the first WS message arrives.
  const setPipelineId = (id: string) => {
    setPipelineIdRaw(id);
    if (id) setPipelineStatus('submitted');
  };

  // Keep a stable reference so the WebSocket handler always sees the latest callback.
  const onCompletedRef = useRef(onCompleted);
  useEffect(() => {
    onCompletedRef.current = onCompleted;
  }, [onCompleted]);

  // Use a ref for pipelineId inside the WS callback to avoid stale closures.
  const pipelineIdRef = useRef(pipelineId);
  useEffect(() => {
    pipelineIdRef.current = pipelineId;
  }, [pipelineId]);

  useWebSocket((msg) => {
    if (
      msg.type === 'pipeline.status' &&
      pipelineIdRef.current &&
      msg.payload.pipeline_id === pipelineIdRef.current
    ) {
      const status = msg.payload.status as string;
      setPipelineStatus(status);
      if (status === 'completed') {
        onCompletedRef.current?.();
      }
    }
  });

  return { pipelineId, pipelineStatus, setPipelineId };
}
