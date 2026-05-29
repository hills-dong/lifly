import UIKit
import PhotosUI
import VisionKit

/// Presents native capture UI (document scanner / photo picker) on behalf of an
/// in-WebView tool and returns base64-encoded JPEGs. Returns nil if cancelled.
@MainActor
final class MediaCapture: NSObject {
    private var continuation: CheckedContinuation<[String]?, Never>?

    nonisolated override init() { super.init() }

    // MARK: Public

    func scanDocument() async -> [String]? {
        guard VNDocumentCameraViewController.isSupported else { return nil }
        return await withCheckedContinuation { cont in
            continuation = cont
            let scanner = VNDocumentCameraViewController()
            scanner.delegate = self
            present(scanner)
        }
    }

    func pickPhotos(max limit: Int) async -> [String]? {
        return await withCheckedContinuation { (cont: CheckedContinuation<[String]?, Never>) in
            continuation = cont
            var config = PHPickerConfiguration()
            config.filter = .images
            config.selectionLimit = Swift.max(1, limit)
            let picker = PHPickerViewController(configuration: config)
            picker.delegate = self
            present(picker)
        }
    }

    // MARK: Helpers

    private func present(_ vc: UIViewController) {
        guard let presenter = Self.topViewController() else {
            finish(nil)
            return
        }
        presenter.present(vc, animated: true)
    }

    private func finish(_ result: [String]?) {
        continuation?.resume(returning: result)
        continuation = nil
    }

    private static func topViewController() -> UIViewController? {
        let scene = UIApplication.shared.connectedScenes
            .compactMap { $0 as? UIWindowScene }
            .first { $0.activationState == .foregroundActive } ?? UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first
        var top = scene?.keyWindow?.rootViewController
            ?? scene?.windows.first(where: { $0.isKeyWindow })?.rootViewController
        while let presented = top?.presentedViewController { top = presented }
        return top
    }

    private static func base64(_ image: UIImage) -> String? {
        image.jpegForUpload()?.base64EncodedString()
    }
}

// MARK: - VisionKit

extension MediaCapture: VNDocumentCameraViewControllerDelegate {
    func documentCameraViewController(
        _ controller: VNDocumentCameraViewController,
        didFinishWith scan: VNDocumentCameraScan
    ) {
        controller.dismiss(animated: true)
        var images: [String] = []
        for i in 0..<scan.pageCount {
            if let b64 = Self.base64(scan.imageOfPage(at: i)) { images.append(b64) }
        }
        finish(images.isEmpty ? nil : images)
    }

    func documentCameraViewControllerDidCancel(_ controller: VNDocumentCameraViewController) {
        controller.dismiss(animated: true)
        finish(nil)
    }

    func documentCameraViewController(
        _ controller: VNDocumentCameraViewController,
        didFailWithError error: Error
    ) {
        controller.dismiss(animated: true)
        finish(nil)
    }
}

// MARK: - PHPicker

extension MediaCapture: PHPickerViewControllerDelegate {
    func picker(_ picker: PHPickerViewController, didFinishPicking results: [PHPickerResult]) {
        picker.dismiss(animated: true)
        guard !results.isEmpty else { finish(nil); return }

        Task {
            var images: [String] = []
            for result in results {
                if let image = await Self.loadImage(from: result.itemProvider),
                   let b64 = Self.base64(image) {
                    images.append(b64)
                }
            }
            finish(images.isEmpty ? nil : images)
        }
    }

    private static func loadImage(from provider: NSItemProvider) async -> UIImage? {
        guard provider.canLoadObject(ofClass: UIImage.self) else { return nil }
        return await withCheckedContinuation { cont in
            provider.loadObject(ofClass: UIImage.self) { object, _ in
                cont.resume(returning: object as? UIImage)
            }
        }
    }
}
