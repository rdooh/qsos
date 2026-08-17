---
session_type: council-design
stage: 5
topic: "Realtime Conversational Streaming vs Asynchronous Batch Production"
date: 2026-08-17
chair: Rob Dooh
facilitator: Antigravity AI
advisors:
  - persona: martin-kleppmann
    name: "Martin Kleppmann"
    domain: "Streaming Architectures & Data Pipelines"
  - persona: dave-farley
    name: "Dave Farley"
    domain: "Low Latency Feedback Loops"
  - persona: rich-hickey
    name: "Rich Hickey"
    domain: "Asynchronous Pipelines & Queueing"
  - persona: steve-jobs
    name: "Steve Jobs"
    domain: "Instant Human Responsiveness"
target_project: qsos
---

# Stage 5 Crucible Council Report: Realtime Conversational Streaming vs Async Batch Production

## 1. Executive Summary

Stage 5 delineates the two primary operational modes of VoiceBox: **Realtime Conversational Streaming** (low-latency, First-Word-Out for active chat) and **Asynchronous Batch Production** (high-quality multi-speaker podcast generation, transcripts, and file rendering).

---

## 2. Dual-Pipeline Architecture

```mermaid
graph TD
    Request["Synthesis Request"] --> PipelineRouter{"Pipeline Router (Mode Selection)"}
    
    PipelineRouter -->|"Conversational Chat Mode"| RealtimePipeline["Realtime Streaming Pipeline (SSE / WebSocket)"]
    PipelineRouter -->|"Batch Production Mode"| BatchPipeline["Async Batch Production Pipeline (Task Queue)"]
    
    RealtimePipeline --> TTFA["First-Word-Out Target: TTFA < 250ms (Kokoro ONNX)"]
    BatchPipeline --> HighFidelity["High-Fidelity Master Rendering (F5-TTS / XTTS-v2 / ElevenLabs)"]
    
    TTFA --> StreamingBuffer["Chunked Audio Stream to Hardware / Client"]
    HighFidelity --> SavedFile["Stitched MP3 / WAV Master File Saved to Disk"]
```

### Mode Comparison Matrix

| Dimension | **Realtime Conversational Streaming** | **Asynchronous Batch Production** |
| :--- | :--- | :--- |
| **Primary Metric** | Time-To-First-Audio (TTFA < 250ms) | Overall Audio Quality, Pitch Smoothness & Stitching |
| **Transport** | SSE (Server-Sent Events) / WebSocket | Async Task Queue / JSON-RPC / File Download |
| **Target Engine** | Kokoro-82M ONNX (Sub-second lightweight) | F5-TTS / XTTS-v2 / ElevenLabs (Heavy transformers) |
| **Use Case** | Active agent voice chat, quick status teasers | Multi-turn council recordings, podcasts, offline listening |

---

## 3. Advisor Consensus & Directives

1. **First-Word-Out Priority**: In conversational mode, streaming chunk generation must start before the entire sentence vector is synthesized.
2. **Explicit Mode Flag**: API parameters must explicitly declare `mode: "conversational"` vs `mode: "batch"` so VoiceBox selects the optimal engine and transport.
