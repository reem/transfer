/// The state of any currently active `http::Stream`.
///
/// Stream States are covered in detail in Section 5.1 of the spec, which also includes
/// a nice diagram showing all of the possible state transitions.
pub enum State {
    /// The "idle" state, as described in the spec:
    ///
    /// ```text
    /// All streams start in the "idle" state.
    ///
    /// The following transitions are valid from this state:
    ///   * Sending or receiving a HEADERS frame causes the stream to become "open". The
    ///     stream identifier is selected as described in Section 5.1.1. The same HEADERS
    ///     frame can also cause the stream to immediately become "half-closed".
    ///   * Sending a PUSH_PROMISE frame on another stream reserves the idle stream that
    ///     is identified for later use. The stream state for the reserved stream transitions to
    ///     "reserved (local)".
    ///   * Receiving a PUSH_PROMISE frame on another stream reserves an idle stream that
    ///     is identified for later use. The stream state for the reserved stream transitions to
    ///     "reserved (remote)".
    ///   * Note that the PUSH_PROMISE frame is not sent on the idle stream but references the
    ///     newly reserved stream in the Promised Stream ID field.
    ///
    /// Receiving any frame other than HEADERS or PRIORITY on a stream in this state MUST be
    /// treated as a connection error of type PROTOCOL_ERROR.
    /// ```
    Idle,

    /// The "reserved (local)" state, as described in the spec:
    ///
    /// ```text
    /// A stream in the "reserved (local)" state is one that has been promised by sending a
    /// PUSH_PROMISE frame. A PUSH_PROMISE frame reserves an idle stream by associating the
    /// stream with an open stream that was initiated by the remote peer (see Section 8.2).
    ///
    /// In this state, only the following transitions are possible:
    ///   * The endpoint can send a HEADERS frame. This causes the stream to open in a "half-
    ///     closed (remote)" state.
    ///   * Either endpoint can send a RST_STREAM frame to cause the stream to become
    ///    "closed". This releases the stream reservation.
    ///
    /// An endpoint MUST NOT send any type of frame other than HEADERS, RST_STREAM, or
    /// PRIORITY in this state.
    ///
    /// A PRIORITY or WINDOW_UPDATE frame MAY be received in this state. Receiving any type of
    /// frame other than RST_STREAM, PRIORITY, or WINDOW_UPDATE on a stream in this state
    /// MUST be treated as a connection error (Section 5.4.1) or type PROTOCOL_ERROR.
    /// ```
    ReservedLocal,

    /// The "reserved (remote)" state, as described in the spec:
    ///
    /// ```text
    /// A stream in the "reserved (remote)" state has been reserved by a remote peer.
    ///
    /// In this state, only the following transitions are possible:
    ///   * Receiving a HEADERS frame causes the stream to transition to "half-closed (local)".
    ///   * Either endpoint can send a RST_STREAM frame to cause the stream to become
    ///     "closed". This releases the stream reservation.
    ///
    /// An endpoint MAY send a PRIORITY frame in this state to reprioritize the reserved stream. An
    /// endpoint MUST NOT send any type of frame other than RST_STREAM, WINDOW_UPDATE, or
    /// PRIORITY in this state.
    ///
    /// Receiving any type of frame other than HEADERS, RST_STREAM, or PRIORITY on a stream in
    /// this state MUST be treated as a connection error (Section 5.4.1) of type PROTOCOL_ERROR.
    /// ```
    ReservedRemote,

    /// The "open" state, as described in the spec:
    ///
    /// ```text
    /// A stream in the "open" state may be used by both peers to send frames of any type. In this
    /// state, sending peers observe advertised stream-level flow-control limits (Section 5.2).
    ///
    /// From this state, either endpoint can send a frame with an END_STREAM flag set, which causes
    /// the stream to transition into one of the "half-closed" states. An endpoint sending an
    /// END_STREAM flag causes the stream state to become "half-closed (local)"; and endpoint
    /// receiving an END_STREAM flag causes the stream to become "half-closed (remote)".
    ///
    /// Either endpoint can send a RST_STREAM frame from this state, causing it to transition
    /// immediately to "closed".
    /// ```
    Open,

    /// The "half-closed (local)" state, as described in the spec:
    ///
    /// ```text
    /// A stream that is in the "half-closed (local)" state cannot be used for sending frames other
    /// than WINDOW_UPDATE, PRIORITY, and RST_STREAM.
    ///
    /// A stream transitions from this state to "closed" when a frame that contains the END_STREAM
    /// flag is received or when either peer sends a RST_STREAM frame.
    ///
    /// An endpoint can receive any type of frame in this state. Providing flow-control credit
    /// using WINDOW_UPDATE frames is necessary to continue receiving flow-controlled frames. In
    /// this state, a receiver can ignore WINDOW_UPDATE frames, which might arrive for a short
    /// period after a frame bearing the END_STREAM flag is sent.
    ///
    /// PRIORITY frames received in this state are used to reprioritize streams that depend on
    /// the identified stream.
    /// ```
    HalfClosedLocal,

    /// The "half-closed (remote)" state, as described in the spec:
    ///
    /// ```text
    /// A stream that is "half-closed (remote)" is no longer being used by the peer to send frames.
    /// In this state, an endpoint is no longer obligated to maintain a receiver flow-control
    /// window.
    ///
    /// If an endpoint receives additional frames other than WINDOW_UPDATE, PRIORITY, or
    /// RST_STREAM, for a stream that is in this state, it MUST respond with a stream error
    /// (Section 5.4.2) of type STREAM_CLOSED.
    ///
    /// A stream that is "half-closed (remote)" can be used by the endpoint to send frames of any
    /// type. In this state, the endpoint continues to observe advertised stream-level flow-control
    /// limits (Section 5.2).
    ///
    /// A stream can transition from this state to "closed" by sending a frame that contains an
    /// END_STREAM flag or when either peer sends a RST_STREAM frame.
    /// ```
    HalfClosedRemote,

    /// The "closed" state, as described in the spec:
    ///
    /// ```text
    /// The "closed" state is the terminal state.
    ///
    /// An endpoint MUST NOT send frames other than PRIORITY on a closed stream. An endpoint that
    /// receives any frame other than PRIORITY after receiving a RST_STREAM MUST treat that as a
    /// stream error (Section 5.4.2) of type STREAM_CLOSED. Similarly, and endpoint that receives
    /// any frames after receiving a frame with the END_STREAM flag set MUST treat that as a
    /// connection error (Section 5.4.2) of type STREAM_CLOSED, unless the frame is permitted as
    /// described below.
    ///
    /// WINDOW_UPDATE or RST_STREAM frames can be received in this state for a short period
    /// after a DATA or HEADERS frame containing an END_STREAM flag is sent. Until the remote peer
    /// receives and processes RST_STREAM or the frame bearing the END_STREAM flag, it might send
    /// frames of these types. Endpoints MUST ignore WINDOW_UPDATE or RST_STREAM frames received in
    /// this state, though endpoints MAY choose to treat frames that arrive a significant time
    /// after sending END_STREAM as a connection error (Section 5.4.1) of type PROTOCOL_ERROR.
    ///
    /// PRIORITY frames can be sent on closed streams to prioritize streams that are dependent on
    /// the closed stream. Endpoints SHOULD process PRIORITY frames, though they can be ignored if
    /// the stream has been removed from the dependency tree (see Section 5.3.4).
    ///
    /// If this state is reached as a result of sending a RST_STREAM frame, the peer that receives
    /// the RST_STREAM might have already sent -- or enqueued for sending -- frames on the stream
    /// that cannot be withdrawn. An endpoint MUST ignore frames it receives on closed streams
    /// after it has sent a RST_STREAM frame. An endpoint MAY choose to limit the period over which
    /// it ignores frames and treates frames that arrive after this time as being in error.
    ///
    /// Flow-controlled frames (i.e., DATA) received after sending RST_STREAM are counted towards
    /// the connection flow-control window. Even though these frames might be ignored, because they
    /// are sent before the sender receives the RST_STREAM, the sender will consider the frames to
    /// count against the flow-control window.
    ///
    /// An endpoint might receive a PUSH_PROMISE frame after it sends RST_STREAM. PUSH_PROMISE
    /// causes a stream to become "reserved" even if the associated stream has been reset.
    /// Therefore, a RST_STREAM is needed to close an unwanted promisd stream.
    /// ```
    Closed
}

