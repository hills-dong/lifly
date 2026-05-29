import Foundation

enum DateFormatting {
    private static let isoWithFraction: ISO8601DateFormatter = {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return f
    }()

    private static let iso: ISO8601DateFormatter = {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime]
        return f
    }()

    private static let dateOnly: DateFormatter = {
        let f = DateFormatter()
        f.locale = Locale(identifier: "en_US_POSIX")
        f.dateFormat = "yyyy-MM-dd"
        return f
    }()

    /// Parse a backend timestamp string into a Date, tolerating several formats.
    static func parse(_ string: String?) -> Date? {
        guard let string, !string.isEmpty else { return nil }
        if let d = isoWithFraction.date(from: string) { return d }
        if let d = iso.date(from: string) { return d }
        if let d = dateOnly.date(from: string) { return d }
        return nil
    }

    private static let display: DateFormatter = {
        let f = DateFormatter()
        f.dateStyle = .medium
        f.timeStyle = .short
        return f
    }()

    private static let displayDateOnly: DateFormatter = {
        let f = DateFormatter()
        f.dateStyle = .medium
        f.timeStyle = .none
        return f
    }()

    /// Friendly display for a timestamp string; falls back to the raw string.
    static func friendly(_ string: String?, dateOnly onlyDate: Bool = false) -> String? {
        guard let string, !string.isEmpty else { return nil }
        guard let date = parse(string) else { return string }
        return (onlyDate ? displayDateOnly : display).string(from: date)
    }
}
