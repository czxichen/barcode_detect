// main.cpp

#include <iostream>
#include <vector>
#include <string>
#include <algorithm>
#include <cmath>
#include <fstream>
#include "ncnn/net.h"
#include "ncnn/simpleocv.h"
#include "ZXing/ImageView.h"
#include "ZXing/ReadBarcode.h"
#include "ZXing/BarcodeFormat.h"

const int NUM_CLASSES = 2;
const int INPUT_IMG_SIZE = 416;
const float NMS_THRESHOLD = 0.5f;

struct Detection
{
    cv::Rect box;
    float score;
    int class_id;
};

struct Decode
{
    Detection detect;
    ZXing::Barcode barcode;
};

std::vector<Decode> DecodeBarcodesInDetections(
    const uint8_t *data, int width, int height, ZXing::ImageFormat format,
    const std::vector<Detection> &boxes)
{
    ZXing::ReaderOptions qrcode_options;
    qrcode_options.setFormats(ZXing::BarcodeFormat::MatrixCodes)
        .setTryInvert(false)
        .setMaxNumberOfSymbols(3);

    ZXing::ReaderOptions barcode_options;
    barcode_options.setFormats(ZXing::BarcodeFormat::LinearCodes)
        .setTryInvert(false)
        .setMaxNumberOfSymbols(3);

    std::vector<Decode> result;
    if (boxes.empty() || data == nullptr)
    {
        return result;
    }

    ZXing::ImageView img(data, width, height, format);

    for (const auto &box : boxes)
    {
        if (box.box.x < 0 || box.box.y < 0 ||
            box.box.x + box.box.width > width ||
            box.box.y + box.box.height > height)
        {
            continue;
        }

        ZXing::ImageView crop = img.cropped(box.box.x, box.box.y, box.box.width, box.box.height);
        const ZXing::ReaderOptions *current_options = nullptr;
        if (box.class_id == 0)
        {
            current_options = &barcode_options;
        }
        else if (box.class_id == 1)
        {
            current_options = &qrcode_options;
        }
        else
        {
            continue;
        }

        auto barcodes = ZXing::ReadBarcodes(crop, *current_options);

        for (const auto &barcode : barcodes)
        {
            Decode decode;
            decode.detect = box;
            decode.barcode = barcode;
            result.push_back(decode);
        }
    }

    return result;
}

/**
 * @brief 将源图像(src)复制到目标图像(dest)的指定区域(ROI)
 * @param src 源图像
 * @param dest 目标图像
 * @param dest_x ROI在目标图像中的左上角x坐标
 * @param dest_y ROI在目标图像中的左上角y坐标
 * @return 如果成功返回 true, 否则返回 false (例如，尺寸超出边界)
 */
bool copy_to_roi(const cv::Mat &src, cv::Mat &dest, int dest_x, int dest_y)
{
    if (!src.data || !dest.data)
    {
        std::cerr << "Error: Source or destination data is null." << std::endl;
        return false;
    }
    if (src.c != dest.c)
    {
        std::cerr << "Error: Channel count mismatch." << std::endl;
        return false;
    }
    if (dest_x < 0 || dest_y < 0 ||
        dest_x + src.cols > dest.cols || dest_y + src.rows > dest.rows)
    {
        std::cerr << "Error: Source image with given offset is out of destination bounds." << std::endl;
        return false;
    }

    // --- 2. 计算步长 (stride) ---
    // 一行像素所占的字节数
    const int src_row_stride = src.cols * src.c;
    const int dest_row_stride = dest.cols * dest.c;

    // --- 3. 逐行复制数据 ---
    // 这种方式比逐像素调用memcpy更高效
    for (int y = 0; y < src.rows; ++y)
    {
        // 计算当前行在源图像中的起始指针
        const unsigned char *src_ptr = src.data + y * src_row_stride;

        // 计算当前行在目标图像中的起始指针
        unsigned char *dest_ptr = dest.data + (dest_y + y) * dest_row_stride + dest_x * dest.c;

        // 使用 memcpy 复制整行数据
        memcpy(dest_ptr, src_ptr, src_row_stride);
    }

    return true;
}
// 计算两个边界框的交并比 (Intersection over Union)
float calculateIoU(const cv::Rect &box1, const cv::Rect &box2)
{
    // 计算交集区域的坐标
    int x_inter1 = std::max(box1.x, box2.x);
    int y_inter1 = std::max(box1.y, box2.y);
    int x_inter2 = std::min(box1.x + box1.width, box2.x + box2.width);
    int y_inter2 = std::min(box1.y + box1.height, box2.y + box2.height);

    // 计算交集区域的宽和高
    int inter_width = std::max(0, x_inter2 - x_inter1);
    int inter_height = std::max(0, y_inter2 - y_inter1);

    // 计算交集面积
    float inter_area = static_cast<float>(inter_width * inter_height);

    // 计算并集面积
    float box1_area = static_cast<float>(box1.width * box1.height);
    float box2_area = static_cast<float>(box2.width * box2.height);
    float union_area = box1_area + box2_area - inter_area;

    // 计算 IoU
    if (union_area == 0)
    {
        return 0.0f; // 避免除以零
    }
    return inter_area / union_area;
}

void NMSBoxes(
    const std::vector<cv::Rect> &boxes,
    const std::vector<float> &scores,
    float score_threshold,
    float nms_threshold,
    std::vector<int> &indices)
{
    indices.clear();

    std::vector<int> temp_indices;
    for (size_t i = 0; i < scores.size(); ++i)
    {
        if (scores[i] >= score_threshold)
        {
            temp_indices.push_back(i);
        }
    }
    std::sort(temp_indices.begin(), temp_indices.end(),
              [&scores](int i1, int i2)
              {
                  return scores[i1] > scores[i2];
              });

    while (!temp_indices.empty())
    {
        int current_idx = temp_indices[0];
        indices.push_back(current_idx);

        const cv::Rect &current_box = boxes[current_idx];

        std::vector<int> remaining_indices;

        for (size_t i = 1; i < temp_indices.size(); ++i)
        {
            int other_idx = temp_indices[i];
            const cv::Rect &other_box = boxes[other_idx];

            float iou = calculateIoU(current_box, other_box);

            if (iou <= nms_threshold)
            {
                remaining_indices.push_back(other_idx);
            }
        }

        temp_indices = remaining_indices;
    }
}

void post_process(const ncnn::Mat &output, std::vector<Detection> &detections, int img_w, int img_h, float score_threshold)
{
    if (output.empty())
    {
        std::cerr << "Warning: Model output is empty. Skipping post-processing." << std::endl;
        return;
    }

    const std::vector<int> strides = {8, 16, 32};
    std::vector<cv::Point2f> grid_centers;
    std::vector<int> expanded_strides;

    for (const auto &stride : strides)
    {
        int feat_w = INPUT_IMG_SIZE / stride;
        int feat_h = INPUT_IMG_SIZE / stride;
        for (int y = 0; y < feat_h; ++y)
        {
            for (int x = 0; x < feat_w; ++x)
            {
                grid_centers.push_back(cv::Point2f(x, y));
                expanded_strides.push_back(stride);
            }
        }
    }

    std::vector<Detection> all_proposals;
    const float *ptr = (float *)output.data;

    for (size_t i = 0; i < grid_centers.size(); ++i)
    {
        const float *cls_scores_ptr = ptr + 5;
        float obj_score = 1.f / (1.f + exp(-ptr[4]));
        int class_id = 0;
        float max_cls_score = 0.f;
        for (int j = 0; j < NUM_CLASSES; ++j)
        {
            float score = 1.f / (1.f + exp(-cls_scores_ptr[j])); // Sigmoid
            if (score > max_cls_score)
            {
                max_cls_score = score;
                class_id = j;
            }
        }

        float final_score = obj_score * max_cls_score;

        if (final_score > score_threshold)
        {
            // --- 解码 BBox ---
            float stride = static_cast<float>(expanded_strides[i]);
            float center_x = (ptr[0] + grid_centers[i].x) * stride;
            float center_y = (ptr[1] + grid_centers[i].y) * stride;
            float w = exp(ptr[2]) * stride;
            float h = exp(ptr[3]) * stride;

            float x1 = center_x - w / 2.f;
            float y1 = center_y - h / 2.f;

            Detection det;
            det.box = cv::Rect(x1, y1, w, h);
            det.score = final_score;
            det.class_id = class_id;
            all_proposals.push_back(det);
        }
        ptr += (5 + NUM_CLASSES); // 移动指针到下一个预测
    }

    // --- 执行 NMS ---
    std::vector<int> indices;
    std::vector<cv::Rect> boxes;
    std::vector<float> scores;
    for (const auto &det : all_proposals)
    {
        boxes.push_back(det.box);
        scores.push_back(det.score);
    }

    NMSBoxes(boxes, scores, score_threshold, NMS_THRESHOLD, indices);

    // 根据原始图像letterbox的padding计算缩放
    float r = std::min(static_cast<float>(INPUT_IMG_SIZE) / img_w, static_cast<float>(INPUT_IMG_SIZE) / img_h);
    int new_w = static_cast<int>(img_w * r);
    int new_h = static_cast<int>(img_h * r);
    int pad_w = (INPUT_IMG_SIZE - new_w) / 2;
    int pad_h = (INPUT_IMG_SIZE - new_h) / 2;

    for (int idx : indices)
    {
        Detection det = all_proposals[idx];

        // 从 letterbox 坐标转换回原始图像坐标
        float x1 = (det.box.x - pad_w) / r;
        float y1 = (det.box.y - pad_h) / r;
        float x2 = (det.box.x + det.box.width - pad_w) / r;
        float y2 = (det.box.y + det.box.height - pad_h) / r;

        // 确保坐标在图像范围内
        x1 = std::max(0.0f, std::min(x1, (float)img_w - 1));
        y1 = std::max(0.0f, std::min(y1, (float)img_h - 1));
        x2 = std::max(0.0f, std::min(x2, (float)img_w - 1));
        y2 = std::max(0.0f, std::min(y2, (float)img_h - 1));

        det.box = cv::Rect(x1, y1, (x2 - x1), (y2 - y1));
        detections.push_back(det);
    }
}

extern "C"
{
    typedef void *DetectorHandle;
    typedef struct
    {
        int class_id;
        int x;
        int y;
        int width;
        int height;
        char *text;
    } DecodeResult;

    typedef struct
    {
        DecodeResult *results;
        int count;
    } DecodeResultList;

    DetectorHandle create_detector(const char *param_mem, const unsigned char *bin_mem)
    {
        ncnn::Net *net = new ncnn::Net();
        if (net->load_param_mem(param_mem) != 0)
        {
            delete net;
            return nullptr;
        }

        if (net->load_model(bin_mem) == 0)
        {
            delete net;
            return nullptr;
        }

        return static_cast<DetectorHandle>(net);
    }
    void destroy_detector(DetectorHandle handle)
    {
        if (handle)
        {
            ncnn::Net *net = static_cast<ncnn::Net *>(handle);
            delete net;
        }
    }

    // 检测置信度阈值
    // const float CONF_THRESHOLD = 0.25f;
    Detection *detect(DetectorHandle handle, const uchar *original_img_ptr, size_t original_img_size, float score_threshold, int *out_detections_count)
    {
        std::vector<uchar> buffer;
        buffer.assign(original_img_ptr, original_img_ptr + original_img_size);

        cv::Mat original_img = cv::imdecode(buffer);

        int img_w = original_img.cols;
        int img_h = original_img.rows;

        float r = std::min(static_cast<float>(INPUT_IMG_SIZE) / img_w, static_cast<float>(INPUT_IMG_SIZE) / img_h);
        int new_w = static_cast<int>(img_w * r);
        int new_h = static_cast<int>(img_h * r);

        cv::Mat resized_img;
        cv::resize(original_img, resized_img, cv::Size(new_w, new_h), 0, 0, 0);

        cv::Mat input_img(INPUT_IMG_SIZE, INPUT_IMG_SIZE, CV_8UC3);
        int pad_w = (INPUT_IMG_SIZE - new_w) / 2;
        int pad_h = (INPUT_IMG_SIZE - new_h) / 2;

        copy_to_roi(resized_img, input_img, pad_w, pad_h);

        ncnn::Mat in = ncnn::Mat::from_pixels(input_img.data, ncnn::Mat::PIXEL_BGR2RGB, INPUT_IMG_SIZE, INPUT_IMG_SIZE);

        const float norm_vals[3] = {1 / 255.f, 1 / 255.f, 1 / 255.f};
        const float mean_vals[3] = {0, 0, 0};
        in.substract_mean_normalize(mean_vals, norm_vals);

        ncnn::Net *net = static_cast<ncnn::Net *>(handle);

        ncnn::Extractor ex = net->create_extractor();
        ex.input("in0", in);

        ncnn::Mat out;
        ex.extract("out0", out);

        std::vector<Detection> detections;
        post_process(out, detections, img_w, img_h, score_threshold);

        *out_detections_count = detections.size();

        if (detections.empty())
        {
            return nullptr;
        }

        Detection *result_arr = new Detection[detections.size()];
        for (size_t i = 0; i < detections.size(); ++i)
        {
            result_arr[i] = detections[i];
        }

        return result_arr;
    }

    Detection *detect_with_pixels(DetectorHandle handle, const uchar *original_img_ptr, int img_s, float score_threshold, int *out_detections_count)
    {
        ncnn::Mat in = in.from_pixels_resize(original_img_ptr, ncnn::Mat::PIXEL_RGB, img_s, img_s, INPUT_IMG_SIZE, INPUT_IMG_SIZE);

        const float norm_vals[3] = {1 / 255.f, 1 / 255.f, 1 / 255.f};
        const float mean_vals[3] = {0, 0, 0};
        in.substract_mean_normalize(mean_vals, norm_vals);

        ncnn::Net *net = static_cast<ncnn::Net *>(handle);

        ncnn::Extractor ex = net->create_extractor();
        ex.input("in0", in);

        ncnn::Mat out;
        ex.extract("out0", out);

        std::vector<Detection> detections;
        post_process(out, detections, img_s, img_s, score_threshold);

        *out_detections_count = detections.size();

        if (detections.empty())
        {
            return nullptr;
        }

        Detection *result_arr = new Detection[detections.size()];
        for (size_t i = 0; i < detections.size(); ++i)
        {
            result_arr[i] = detections[i];
        }

        return result_arr;
    }

    void detect_result_free(Detection *detections)
    {
        if (detections)
        {
            delete[] detections;
        }
    }

    DecodeResultList decode_detections(const unsigned char *data, int width, int height, const Detection *detections, int detections_size)
    {
        std::vector<Detection> boxes(detections, detections + detections_size);
        std::vector<Decode> decodes = DecodeBarcodesInDetections(data, width, height, ZXing::ImageFormat::RGB, boxes);

        DecodeResultList result;
        result.results = nullptr;
        result.count = decodes.size();
        if (result.count == 0)
        {
            return result;
        }

        result.results = new DecodeResult[result.count];

        for (int i = 0; i < result.count; ++i)
        {
            const auto &decode = decodes[i];
            DecodeResult &c_result = result.results[i];

            c_result.x = decode.detect.box.x;
            c_result.y = decode.detect.box.y;
            c_result.width = decode.detect.box.width;
            c_result.height = decode.detect.box.height;

            c_result.class_id = static_cast<int>(decode.barcode.format());
            const std::string &text_str = decode.barcode.text();
            c_result.text = new char[text_str.length() + 1];
            strcpy(c_result.text, text_str.c_str());
        }

        return result;
    }

    void decode_result_free(DecodeResultList *result_list)
    {
        if (result_list == nullptr || result_list->results == nullptr)
        {
            return;
        }

        for (int i = 0; i < result_list->count; ++i)
        {
            delete[] result_list->results[i].text;
        }

        delete[] result_list->results;

        result_list->results = nullptr;
        result_list->count = 0;
    }
}