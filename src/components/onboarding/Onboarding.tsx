import React, { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { platform } from "@tauri-apps/plugin-os";
import { toast } from "sonner";
import { Check, Download, Loader2 } from "lucide-react";
import type { ModelInfo } from "@/bindings";
import type { ModelCardStatus } from "./ModelCard";
import ModelCard from "./ModelCard";
import { useModelStore } from "../../stores/modelStore";
import { formatModelSize } from "../../lib/utils/format";

interface OnboardingProps {
  onModelSelected: () => void;
}

const LLM_MODEL_ID = "qwen-2.5-1.5b";

const Onboarding: React.FC<OnboardingProps> = ({ onModelSelected }) => {
  const { t } = useTranslation();
  const {
    models,
    downloadModel,
    selectModel,
    downloadingModels,
    extractingModels,
    downloadProgress,
    downloadStats,
  } = useModelStore();
  const [selectedSttModelId, setSelectedSttModelId] = useState<string | null>(
    null,
  );
  const [downloadStarted, setDownloadStarted] = useState(false);

  const isMacOS = platform() === "macos";

  // Recommended STT model (Parakeet V3) — find even if already downloaded
  const recommendedSttModel = useMemo(
    () =>
      models.find(
        (m) => m.is_recommended && m.engine_type !== "LocalLlm",
      ),
    [models],
  );

  // LLM model (only relevant on macOS)
  const llmModel = useMemo(
    () => models.find((m) => m.id === LLM_MODEL_ID),
    [models],
  );

  const llmNeedsDownload =
    isMacOS && llmModel && !llmModel.is_downloaded;

  // Combined size for the recommended download button
  const recommendedTotalSize = useMemo(() => {
    let total = 0;
    if (recommendedSttModel && !recommendedSttModel.is_downloaded)
      total += Number(recommendedSttModel.size_mb);
    if (llmNeedsDownload) total += Number(llmModel.size_mb);
    return total;
  }, [recommendedSttModel, llmModel, llmNeedsDownload]);

  // Alternative STT models (non-recommended, non-LLM, not downloaded)
  const alternativeModels = useMemo(
    () =>
      models
        .filter(
          (m: ModelInfo) =>
            !m.is_downloaded &&
            !m.is_recommended &&
            m.engine_type !== "LocalLlm",
        )
        .sort(
          (a: ModelInfo, b: ModelInfo) =>
            Number(a.size_mb) - Number(b.size_mb),
        ),
    [models],
  );

  // Track whether any download is in progress
  const isDownloading = selectedSttModelId !== null || downloadStarted;

  // Advance once ALL required models are downloaded (STT + LLM on macOS)
  useEffect(() => {
    if (!selectedSttModelId) return;

    const sttModel = models.find((m) => m.id === selectedSttModelId);
    const sttReady =
      sttModel?.is_downloaded &&
      !(selectedSttModelId in downloadingModels) &&
      !(selectedSttModelId in extractingModels);

    // On macOS, also wait for LLM model to finish
    const llmReady = !isMacOS || !llmModel || (
      llmModel.is_downloaded &&
      !(LLM_MODEL_ID in downloadingModels) &&
      !(LLM_MODEL_ID in extractingModels)
    );

    if (sttReady && llmReady) {
      selectModel(selectedSttModelId).then((success) => {
        if (success) {
          onModelSelected();
        } else {
          toast.error(t("onboarding.errors.selectModel"));
          setSelectedSttModelId(null);
          setDownloadStarted(false);
        }
      });
    }
  }, [
    selectedSttModelId,
    models,
    downloadingModels,
    extractingModels,
    selectModel,
    onModelSelected,
    isMacOS,
    llmModel,
    t,
  ]);

  // Download recommended models (STT + LLM on macOS)
  const handleDownloadRecommended = async () => {
    if (!recommendedSttModel) return;

    setSelectedSttModelId(recommendedSttModel.id);
    setDownloadStarted(true);

    // Start LLM download in parallel
    if (llmNeedsDownload) {
      downloadModel(LLM_MODEL_ID);
    }

    // Start the STT download
    const success = await downloadModel(recommendedSttModel.id);
    if (!success) {
      toast.error(t("onboarding.downloadFailed"));
      setSelectedSttModelId(null);
      setDownloadStarted(false);
    }
    // Advancement happens in the useEffect once both models are ready
  };

  // Download an alternative STT model (+ LLM on macOS)
  const handleDownloadAlternative = async (modelId: string) => {
    setSelectedSttModelId(modelId);
    setDownloadStarted(true);

    // Start LLM download in parallel
    if (isMacOS && llmModel && !llmModel.is_downloaded) {
      downloadModel(LLM_MODEL_ID);
    }

    // Start the STT download
    const success = await downloadModel(modelId);
    if (!success) {
      toast.error(t("onboarding.downloadFailed"));
      setSelectedSttModelId(null);
      setDownloadStarted(false);
    }
    // Advancement happens in the useEffect once both models are ready
  };

  const getModelStatus = (modelId: string): ModelCardStatus => {
    if (modelId in extractingModels) return "extracting";
    if (modelId in downloadingModels) return "downloading";
    const model = models.find((m) => m.id === modelId);
    if (model?.is_downloaded) return "available";
    return "downloadable";
  };

  const getModelDownloadProgress = (modelId: string): number | undefined => {
    return downloadProgress[modelId]?.percentage;
  };

  const getModelDownloadSpeed = (modelId: string): number | undefined => {
    return downloadStats[modelId]?.speed;
  };

  const recommendedSttStatus = recommendedSttModel
    ? getModelStatus(recommendedSttModel.id)
    : "downloadable";
  const llmStatus = llmModel ? getModelStatus(llmModel.id) : "downloadable";

  // Show recommended section when download hasn't started yet, or while any download is active
  const showRecommendedSection =
    recommendedSttModel &&
    (recommendedSttStatus === "downloadable" ||
      recommendedSttStatus === "downloading" ||
      recommendedSttStatus === "extracting" ||
      llmStatus === "downloading" ||
      llmStatus === "extracting");

  // Show download button only before any download has started
  const showDownloadButton =
    !downloadStarted &&
    recommendedSttStatus === "downloadable";

  return (
    <div className="h-screen w-screen flex flex-col p-6 gap-4 inset-0 items-center">
      <div className="flex flex-col items-center gap-2 shrink-0">
        <p className="text-text/70 max-w-md font-medium mx-auto text-center">
          {t("onboarding.subtitle")}
        </p>
      </div>

      <div className="max-w-[600px] w-full mx-auto text-center flex-1 flex flex-col min-h-0 overflow-y-auto">
        <div className="flex flex-col gap-6 pb-6">
          {/* Recommended models section */}
          {showRecommendedSection && (
            <div className="border-2 border-logo-primary/25 rounded-xl p-4 flex flex-col gap-3">
              <h3 className="text-sm font-semibold text-text/60 uppercase tracking-wide text-left">
                {t("onboarding.recommendedSection")}
              </h3>

              {/* Recommended STT model compact card */}
              <RecommendedModelRow
                model={recommendedSttModel}
                label={t("onboarding.transcriptionModel")}
                status={recommendedSttStatus}
                downloadProgress={getModelDownloadProgress(
                  recommendedSttModel.id,
                )}
                downloadSpeed={getModelDownloadSpeed(recommendedSttModel.id)}
              />

              {/* LLM model compact card (macOS only) */}
              {isMacOS && llmModel && (
                <RecommendedModelRow
                  model={llmModel}
                  label={t("onboarding.postProcessingModel")}
                  status={llmStatus}
                  downloadProgress={getModelDownloadProgress(LLM_MODEL_ID)}
                  downloadSpeed={getModelDownloadSpeed(LLM_MODEL_ID)}
                />
              )}

              {/* Single download button for both */}
              {showDownloadButton && (
                <button
                  type="button"
                  onClick={handleDownloadRecommended}
                  disabled={isDownloading}
                  className="mt-1 w-full flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-logo-primary text-white font-medium text-sm hover:bg-logo-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Download className="w-4 h-4" />
                  {t("onboarding.downloadRecommended")}
                  <span className="text-white/70 ml-1">
                    {t("onboarding.totalSize", {
                      size: formatModelSize(recommendedTotalSize),
                    })}
                  </span>
                </button>
              )}
            </div>
          )}

          {/* Alternative models section */}
          {alternativeModels.length > 0 && (
            <>
              <p className="text-sm text-text/50 font-medium">
                {t("onboarding.alternativeSection")}
              </p>
              {alternativeModels.map((model: ModelInfo) => (
                <ModelCard
                  key={model.id}
                  model={model}
                  status={getModelStatus(model.id)}
                  disabled={isDownloading}
                  onSelect={handleDownloadAlternative}
                  onDownload={handleDownloadAlternative}
                  downloadProgress={getModelDownloadProgress(model.id)}
                  downloadSpeed={getModelDownloadSpeed(model.id)}
                  showRecommended={false}
                />
              ))}
            </>
          )}
        </div>
      </div>
    </div>
  );
};

/** Compact row for a recommended model inside the bordered section */
function RecommendedModelRow({
  model,
  label,
  status,
  downloadProgress,
  downloadSpeed,
}: {
  model: ModelInfo;
  label: string;
  status: ModelCardStatus;
  downloadProgress?: number;
  downloadSpeed?: number;
}) {
  const { t } = useTranslation();

  const isComplete = status === "available";

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-text">{model.name}</span>
          <span className="text-xs text-text/50">{label}</span>
        </div>
        {isComplete ? (
          <Check className="w-4 h-4 text-logo-primary" />
        ) : (
          <span className="text-xs text-text/50">
            {formatModelSize(Number(model.size_mb))}
          </span>
        )}
      </div>

      {/* Download progress */}
      {status === "downloading" && downloadProgress !== undefined && (
        <div className="w-full">
          <div className="w-full h-1.5 bg-mid-gray/20 rounded-full overflow-hidden">
            <div
              className="h-full bg-logo-primary rounded-full transition-all duration-300"
              style={{ width: `${downloadProgress}%` }}
            />
          </div>
          <div className="flex items-center justify-between text-xs mt-0.5">
            <span className="text-text/50">
              {t("modelSelector.downloading", {
                percentage: Math.round(downloadProgress),
              })}
            </span>
            {downloadSpeed !== undefined && downloadSpeed > 0 && (
              <span className="tabular-nums text-text/50">
                {t("modelSelector.downloadSpeed", {
                  speed: downloadSpeed.toFixed(1),
                })}
              </span>
            )}
          </div>
        </div>
      )}

      {/* Extraction progress */}
      {status === "extracting" && (
        <div className="w-full">
          <div className="w-full h-1.5 bg-mid-gray/20 rounded-full overflow-hidden">
            <div className="h-full bg-logo-primary rounded-full animate-pulse w-full" />
          </div>
          <div className="flex items-center gap-1 text-xs text-text/50 mt-0.5">
            <Loader2 className="w-3 h-3 animate-spin" />
            <span>{t("modelSelector.extractingGeneric")}</span>
          </div>
        </div>
      )}
    </div>
  );
}

export default Onboarding;
