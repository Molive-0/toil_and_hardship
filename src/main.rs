#![no_main]
#![no_std]
#![windows_subsystem = "windows"]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]

mod miniwin;
mod util;
mod vk;
use miniwin::handle_message;
use vk::*;
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleA;

use core::{ffi::c_void, mem::MaybeUninit, panic::PanicInfo, ptr};

#[panic_handler]
#[no_mangle]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

const DEVICE_FEATURES: PhysicalDeviceFeatures = PhysicalDeviceFeatures {
    robustBufferAccess: FALSE,
    fullDrawIndexUint32: FALSE,
    imageCubeArray: FALSE,
    independentBlend: FALSE,
    geometryShader: FALSE,
    tessellationShader: FALSE,
    sampleRateShading: FALSE,
    dualSrcBlend: FALSE,
    logicOp: FALSE,
    multiDrawIndirect: FALSE,
    drawIndirectFirstInstance: FALSE,
    depthClamp: FALSE,
    depthBiasClamp: FALSE,
    fillModeNonSolid: FALSE,
    depthBounds: FALSE,
    wideLines: FALSE,
    largePoints: FALSE,
    alphaToOne: FALSE,
    multiViewport: FALSE,
    samplerAnisotropy: FALSE,
    textureCompressionETC2: FALSE,
    textureCompressionASTC_LDR: FALSE,
    textureCompressionBC: FALSE,
    occlusionQueryPrecise: FALSE,
    pipelineStatisticsQuery: FALSE,
    vertexPipelineStoresAndAtomics: FALSE,
    fragmentStoresAndAtomics: FALSE,
    shaderTessellationAndGeometryPointSize: FALSE,
    shaderImageGatherExtended: FALSE,
    shaderStorageImageExtendedFormats: FALSE,
    shaderStorageImageMultisample: FALSE,
    shaderStorageImageReadWithoutFormat: FALSE,
    shaderStorageImageWriteWithoutFormat: FALSE,
    shaderUniformBufferArrayDynamicIndexing: FALSE,
    shaderSampledImageArrayDynamicIndexing: FALSE,
    shaderStorageBufferArrayDynamicIndexing: FALSE,
    shaderStorageImageArrayDynamicIndexing: FALSE,
    shaderClipDistance: FALSE,
    shaderCullDistance: FALSE,
    shaderFloat64: FALSE,
    shaderInt64: FALSE,
    shaderInt16: FALSE,
    shaderResourceResidency: FALSE,
    shaderResourceMinLod: FALSE,
    sparseBinding: FALSE,
    sparseResidencyBuffer: FALSE,
    sparseResidencyImage2D: FALSE,
    sparseResidencyImage3D: FALSE,
    sparseResidency2Samples: FALSE,
    sparseResidency4Samples: FALSE,
    sparseResidency8Samples: FALSE,
    sparseResidency16Samples: FALSE,
    sparseResidencyAliased: FALSE,
    variableMultisampleRate: FALSE,
    inheritedQueries: FALSE,
};

macro_rules! create_shader_module {
    ($s:expr, $ps:ident, $device:ident) => {{
        let shader = include_bytes!($s);
        let create_info = ShaderModuleCreateInfo {
            sType: STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
            codeSize: shader.len(),
            pCode: shader.as_ptr() as *const u32,
            pNext: ptr::null(),
            flags: 0,
        };
        unsafe {
            let mut shader = MaybeUninit::uninit();
            $ps.CreateShaderModule($device, &create_info, ptr::null(), shader.as_mut_ptr());
            shader.assume_init()
        }
    }};
}

fn init_vulkan(window: HWND, ps: &Static) {
    let instance = create_instance(ps);
    let surface = create_surface(ps, window, instance);
    let physical_device = pick_physical_device(ps, instance);
    let (device, queue) = create_logical_device(ps, physical_device);
    let (swapchain, image) = create_swapchain(ps, device, surface);
    let image_view = create_image_view(ps, device, image);
    let render_pass = create_render_pass(ps, device);
    let pipeline = create_graphics_pipeline(ps, device, render_pass);
    let framebuffer = create_framebuffers(ps, device, image_view, render_pass);
    let command_pool = create_command_pool(ps, device);
    let command_buffer =
        create_command_buffers(ps, device, command_pool, pipeline, render_pass, framebuffer);
    let (available, rendered, fence) = create_sync_objects(ps, device);
    draw_frame(
        ps,
        device,
        swapchain,
        command_buffer,
        queue,
        fence,
        available,
        rendered,
    )
}

fn create_instance(ps: &Static) -> Instance {
    const APP_INFO: ApplicationInfo = ApplicationInfo {
        sType: STRUCTURE_TYPE_APPLICATION_INFO,
        pApplicationName: "\0".as_ptr() as *const i8,
        applicationVersion: 0,
        pEngineName: "\0".as_ptr() as *const i8,
        engineVersion: 0,
        apiVersion: 0b_000_0000001_0000000010_000000000000,
        pNext: ptr::null(),
    };

    const EXTENSIONS: [*const i8; 2] = [
        "VK_KHR_surface\0".as_ptr() as *const i8,
        "VK_KHR_win32_surface\0".as_ptr() as *const i8,
    ];

    const CREATE_INFO: InstanceCreateInfo = InstanceCreateInfo {
        sType: STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        pApplicationInfo: &APP_INFO,
        enabledExtensionCount: EXTENSIONS.len() as u32,
        ppEnabledExtensionNames: EXTENSIONS.as_ptr(),
        enabledLayerCount: 0,
        ppEnabledLayerNames: ptr::null(),
        pNext: ptr::null(),
        flags: 0,
    };

    let mut instance = MaybeUninit::uninit();
    unsafe {
        ps.CreateInstance(&CREATE_INFO, ptr::null(), instance.as_mut_ptr());
        instance.assume_init()
    }
}

fn create_surface(ps: &Static, hwnd: HWND, instance: Instance) -> SurfaceKHR {
    let create_info = Win32SurfaceCreateInfoKHR {
        sType: STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
        hwnd: (hwnd as *mut _ as *mut c_void),
        hinstance: unsafe { GetModuleHandleA(ptr::null()) as *mut _ as *mut c_void },
        pNext: ptr::null(),
        flags: 0,
    };

    let mut surface = MaybeUninit::uninit();
    unsafe {
        ps.CreateWin32SurfaceKHR(instance, &create_info, ptr::null(), surface.as_mut_ptr());
        surface.assume_init()
    }
}

fn pick_physical_device(ps: &Static, instance: Instance) -> PhysicalDevice {
    //bounds checks are for losers, just don't plug in more than 8 gpus ok
    let mut devices: [MaybeUninit<PhysicalDevice>; 8] = MaybeUninit::uninit_array();
    let mut _count = MaybeUninit::uninit();
    unsafe {
        ps.EnumeratePhysicalDevices(
            instance,
            _count.as_mut_ptr(),
            devices.as_mut_ptr() as *mut PhysicalDevice,
        );
        devices[0].assume_init()
    }
}

fn create_logical_device(ps: &Static, physical_device: PhysicalDevice) -> (Device, Queue) {
    // Here we take all infos to be const values. We can do this because most
    // vendor cards will provide family 0 as graphics and present, and so
    // this code assumes as such.
    const QUEUE_CREATE_INFO: DeviceQueueCreateInfo = DeviceQueueCreateInfo {
        sType: STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
        queueFamilyIndex: 0,
        queueCount: 1,
        pQueuePriorities: &1.0,
        pNext: ptr::null(),
        flags: 0,
    };
    const DEVICE_EXTENSIONS: [*const i8; 1] = ["VK_KHR_swapchain\0".as_ptr() as *const i8];
    const CREATE_INFO: DeviceCreateInfo = DeviceCreateInfo {
        sType: STRUCTURE_TYPE_DEVICE_CREATE_INFO,
        pQueueCreateInfos: &QUEUE_CREATE_INFO,
        queueCreateInfoCount: 1,
        enabledLayerCount: 0,
        ppEnabledLayerNames: ptr::null(),
        ppEnabledExtensionNames: DEVICE_EXTENSIONS.as_ptr(),
        enabledExtensionCount: DEVICE_EXTENSIONS.len() as u32,
        pEnabledFeatures: &DEVICE_FEATURES,
        pNext: ptr::null(),
        flags: 0,
    };
    let mut device = MaybeUninit::uninit();
    let mut queue = MaybeUninit::uninit();
    let device = unsafe {
        ps.CreateDevice(
            physical_device,
            &CREATE_INFO,
            ptr::null(),
            device.as_mut_ptr(),
        );
        device.assume_init()
    };
    unsafe {
        ps.GetDeviceQueue(device, 0, 0, queue.as_mut_ptr());
    }
    unsafe { (device, queue.assume_init()) }
}

fn create_swapchain(ps: &Static, device: Device, surface: SurfaceKHR) -> (SwapchainKHR, Image) {
    let create_info;
    const CREATE_INFO: SwapchainCreateInfoKHR = SwapchainCreateInfoKHR {
        sType: STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
        surface: 0,
        minImageCount: 1,
        imageFormat: FORMAT_R8G8B8A8_SRGB,
        imageColorSpace: COLOR_SPACE_SRGB_NONLINEAR_KHR,
        imageExtent: Extent2D {
            width: 1920,
            height: 1080,
        },
        presentMode: PRESENT_MODE_IMMEDIATE_KHR,
        imageArrayLayers: 1,
        imageUsage: IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
        imageSharingMode: SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 1,
        pQueueFamilyIndices: &0,
        preTransform: SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
        compositeAlpha: COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
        clipped: TRUE,
        oldSwapchain: NULL_HANDLE,
        pNext: ptr::null(),
        flags: 0,
    };
    let mut create_info_i = CREATE_INFO;
    create_info_i.surface = surface;
    create_info = create_info_i;
    let swapchain: SwapchainKHR = unsafe {
        let mut swapchain = MaybeUninit::uninit();
        ps.CreateSwapchainKHR(device, &create_info, ptr::null(), swapchain.as_mut_ptr());
        swapchain.assume_init()
    };
    let image: Image = unsafe {
        let mut image = MaybeUninit::uninit();
        let mut count = MaybeUninit::uninit();
        ps.GetSwapchainImagesKHR(device, swapchain, count.as_mut_ptr(), image.as_mut_ptr());
        //let count = count.assume_init();
        //if count > 1 {
        //    show_error("Count is greater than one\0".as_ptr() as *const i8);
        //}
        image.assume_init()
    };
    (swapchain, image)
}

fn create_image_view(ps: &Static, device: Device, image: Image) -> ImageView {
    let create_info;
    {
        const CREATE_INFO: ImageViewCreateInfo = ImageViewCreateInfo {
            sType: STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
            image: 0,
            viewType: IMAGE_VIEW_TYPE_2D,
            format: FORMAT_R8G8B8A8_SRGB,
            components: ComponentMapping {
                r: COMPONENT_SWIZZLE_IDENTITY,
                g: COMPONENT_SWIZZLE_IDENTITY,
                b: COMPONENT_SWIZZLE_IDENTITY,
                a: COMPONENT_SWIZZLE_IDENTITY,
            },
            subresourceRange: ImageSubresourceRange {
                aspectMask: IMAGE_ASPECT_COLOR_BIT,
                baseMipLevel: 0,
                levelCount: 1,
                baseArrayLayer: 0,
                layerCount: 1,
            },
            pNext: ptr::null(),
            flags: 0,
        };
        let mut create_info_i = CREATE_INFO;
        create_info_i.image = image;
        create_info = create_info_i;
    }
    unsafe {
        let mut image_view = MaybeUninit::uninit();
        ps.CreateImageView(device, &create_info, ptr::null(), image_view.as_mut_ptr());
        image_view.assume_init()
    }
}

fn create_render_pass(ps: &Static, device: Device) -> RenderPass {
    const COLOR_ATTACHMENT: AttachmentDescription = AttachmentDescription {
        format: FORMAT_R8G8B8A8_SRGB,
        samples: SAMPLE_COUNT_1_BIT,
        loadOp: ATTACHMENT_LOAD_OP_CLEAR,
        storeOp: ATTACHMENT_STORE_OP_STORE,
        stencilLoadOp: ATTACHMENT_LOAD_OP_DONT_CARE,
        stencilStoreOp: ATTACHMENT_STORE_OP_DONT_CARE,
        initialLayout: IMAGE_LAYOUT_UNDEFINED,
        finalLayout: IMAGE_LAYOUT_PRESENT_SRC_KHR,
        flags: 0,
    };
    const COLOR_ATTACHMENT_REF: AttachmentReference = AttachmentReference {
        attachment: 0,
        layout: IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
    };
    const SUBPASS: SubpassDescription = SubpassDescription {
        pipelineBindPoint: PIPELINE_BIND_POINT_GRAPHICS,
        colorAttachmentCount: 1,
        pColorAttachments: &COLOR_ATTACHMENT_REF,
        inputAttachmentCount: 0,
        pInputAttachments: ptr::null(),
        preserveAttachmentCount: 0,
        pPreserveAttachments: ptr::null(),
        pResolveAttachments: ptr::null(),
        pDepthStencilAttachment: ptr::null(),
        flags: 0,
    };
    const DEPENDENCY: SubpassDependency = SubpassDependency {
        srcSubpass: SUBPASS_EXTERNAL,
        dstSubpass: 0,
        srcStageMask: PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
        srcAccessMask: 0,
        dstStageMask: PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
        dstAccessMask: ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
        dependencyFlags: 0,
    };
    const RENDER_PASS_INFO: RenderPassCreateInfo = RenderPassCreateInfo {
        sType: STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
        attachmentCount: 1,
        pAttachments: &COLOR_ATTACHMENT,
        subpassCount: 1,
        pSubpasses: &SUBPASS,
        dependencyCount: 1,
        pDependencies: &DEPENDENCY,
        pNext: ptr::null(),
        flags: 0,
    };
    unsafe {
        let mut render_pass = MaybeUninit::uninit();
        ps.CreateRenderPass(
            device,
            &RENDER_PASS_INFO,
            ptr::null(),
            render_pass.as_mut_ptr(),
        );
        render_pass.assume_init()
    }
}

fn create_graphics_pipeline(ps: &Static, device: Device, render_pass: RenderPass) -> Pipeline {
    let vert_shader_module = create_shader_module!("../shaders/vert.spv", ps, device);
    let frag_shader_module = create_shader_module!("../shaders/frag.spv", ps, device);

    const STAGE_INFO: PipelineShaderStageCreateInfo = PipelineShaderStageCreateInfo {
        sType: STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
        stage: SHADER_STAGE_VERTEX_BIT,
        module: 0,
        pName: "main\0".as_ptr() as *const i8,
        pSpecializationInfo: ptr::null(),
        pNext: ptr::null(),
        flags: 0,
    };

    let mut vert_shader_stage_info = STAGE_INFO;
    vert_shader_stage_info.module = vert_shader_module;
    let vert_shader_stage_info = vert_shader_stage_info;

    let mut frag_shader_stage_info = STAGE_INFO;
    frag_shader_stage_info.module = frag_shader_module;
    frag_shader_stage_info.stage = SHADER_STAGE_FRAGMENT_BIT;
    let frag_shader_stage_info = frag_shader_stage_info;

    let shader_stages = [vert_shader_stage_info, frag_shader_stage_info];

    const VERTEX_INPUT_INFO: PipelineVertexInputStateCreateInfo =
        PipelineVertexInputStateCreateInfo {
            sType: STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            vertexBindingDescriptionCount: 0,
            vertexAttributeDescriptionCount: 0,
            pVertexBindingDescriptions: ptr::null(),
            pVertexAttributeDescriptions: ptr::null(),
            pNext: ptr::null(),
            flags: 0,
        };
    const INPUT_ASSEMBLY: PipelineInputAssemblyStateCreateInfo =
        PipelineInputAssemblyStateCreateInfo {
            sType: STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            topology: PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
            primitiveRestartEnable: FALSE,
            pNext: ptr::null(),
            flags: 0,
        };
    const VIEWPORT: Viewport = Viewport {
        x: 0f32,
        y: 0f32,
        width: 1920f32,
        height: 1080f32,
        minDepth: 0f32,
        maxDepth: 1f32,
    };
    const SCISSOR: Rect2D = Rect2D {
        offset: Offset2D { x: 0, y: 0 },
        extent: Extent2D {
            width: 1920,
            height: 1080,
        },
    };
    const VIEWPORT_STATE: PipelineViewportStateCreateInfo = PipelineViewportStateCreateInfo {
        sType: STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        viewportCount: 1,
        pViewports: &VIEWPORT,
        scissorCount: 1,
        pScissors: &SCISSOR,
        pNext: ptr::null(),
        flags: 0,
    };
    const RASTERISER: PipelineRasterizationStateCreateInfo = PipelineRasterizationStateCreateInfo {
        sType: STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        depthClampEnable: FALSE,
        depthBiasEnable: FALSE,
        depthBiasClamp: unsafe { core::mem::transmute(0u32) },
        depthBiasConstantFactor: unsafe { core::mem::transmute(0u32) },
        depthBiasSlopeFactor: unsafe { core::mem::transmute(0u32) },
        rasterizerDiscardEnable: FALSE,
        polygonMode: POLYGON_MODE_FILL,
        lineWidth: 1f32,
        cullMode: CULL_MODE_BACK_BIT,
        frontFace: FRONT_FACE_CLOCKWISE,
        pNext: ptr::null(),
        flags: 0,
    };
    const MULTISAMPLING: PipelineMultisampleStateCreateInfo = PipelineMultisampleStateCreateInfo {
        sType: STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        sampleShadingEnable: FALSE,
        rasterizationSamples: SAMPLE_COUNT_1_BIT,
        minSampleShading: unsafe { core::mem::transmute(0u32) },
        alphaToCoverageEnable: FALSE,
        alphaToOneEnable: FALSE,
        pSampleMask: ptr::null(),
        pNext: ptr::null(),
        flags: 0,
    };
    const COLOR_BLEND_ATTACHMENT: PipelineColorBlendAttachmentState =
        PipelineColorBlendAttachmentState {
            colorWriteMask: COLOR_COMPONENT_R_BIT
                | COLOR_COMPONENT_G_BIT
                | COLOR_COMPONENT_B_BIT
                | COLOR_COMPONENT_A_BIT,
            blendEnable: FALSE,
            colorBlendOp: FALSE,
            dstColorBlendFactor: FALSE,
            srcColorBlendFactor: FALSE,
            alphaBlendOp: FALSE,
            dstAlphaBlendFactor: FALSE,
            srcAlphaBlendFactor: FALSE,
        };
    const COLOR_BLENDING: PipelineColorBlendStateCreateInfo = PipelineColorBlendStateCreateInfo {
        sType: STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        logicOpEnable: FALSE,
        logicOp: LOGIC_OP_COPY,
        attachmentCount: 1,
        pAttachments: &COLOR_BLEND_ATTACHMENT,
        blendConstants: [0f32, 0f32, 0f32, 0f32],
        pNext: ptr::null(),
        flags: 0,
    };
    const PIPELINE_LAYOUT_INFO: PipelineLayoutCreateInfo = PipelineLayoutCreateInfo {
        sType: STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
        setLayoutCount: 0,
        pushConstantRangeCount: 0,
        pPushConstantRanges: ptr::null(),
        pSetLayouts: ptr::null(),
        pNext: ptr::null(),
        flags: 0,
    };
    let pipeline_layout = unsafe {
        let mut pipeline_layout = MaybeUninit::uninit();
        ps.CreatePipelineLayout(
            device,
            &PIPELINE_LAYOUT_INFO,
            ptr::null(),
            pipeline_layout.as_mut_ptr(),
        );
        pipeline_layout.assume_init()
    };
    const PIPELINE_INFO: GraphicsPipelineCreateInfo = GraphicsPipelineCreateInfo {
        sType: STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
        stageCount: 2,
        pStages: ptr::null(),
        pVertexInputState: &VERTEX_INPUT_INFO,
        pInputAssemblyState: &INPUT_ASSEMBLY,
        pViewportState: &VIEWPORT_STATE,
        pRasterizationState: &RASTERISER,
        pMultisampleState: &MULTISAMPLING,
        pColorBlendState: &COLOR_BLENDING,
        layout: 0,
        renderPass: 0,
        subpass: 0,
        basePipelineHandle: NULL_HANDLE,
        basePipelineIndex: 0,
        pDepthStencilState: ptr::null(),
        pDynamicState: ptr::null(),
        pTessellationState: ptr::null(),
        pNext: ptr::null(),
        flags: 0,
    };
    let mut pipeline_info = PIPELINE_INFO;
    pipeline_info.pStages = shader_stages.as_ptr();
    pipeline_info.layout = pipeline_layout;
    pipeline_info.renderPass = render_pass;
    let pipeline_info = pipeline_info;

    unsafe {
        let mut pipeline = MaybeUninit::uninit();
        ps.CreateGraphicsPipelines(
            device,
            NULL_HANDLE,
            1,
            &pipeline_info,
            ptr::null(),
            pipeline.as_mut_ptr(),
        );
        pipeline.assume_init()
    }
}

fn create_framebuffers(
    ps: &Static,
    device: Device,
    image_view: ImageView,
    render_pass: RenderPass,
) -> Framebuffer {
    const FRAMEBUFFER_INFO: FramebufferCreateInfo = FramebufferCreateInfo {
        sType: STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
        renderPass: NULL_HANDLE,
        attachmentCount: 1,
        pAttachments: ptr::null(),
        width: 1920,
        height: 1080,
        layers: 1,
        pNext: ptr::null(),
        flags: 0,
    };
    let mut framebuffer_info = FRAMEBUFFER_INFO;
    framebuffer_info.pAttachments = &image_view;
    framebuffer_info.renderPass = render_pass;
    let framebuffer_info = framebuffer_info;

    unsafe {
        let mut framebuffer = MaybeUninit::uninit();
        ps.CreateFramebuffer(
            device,
            &framebuffer_info,
            ptr::null(),
            framebuffer.as_mut_ptr(),
        );
        framebuffer.assume_init()
    }
}

fn create_command_pool(ps: &Static, device: Device) -> CommandPool {
    const POOL_INFO: CommandPoolCreateInfo = CommandPoolCreateInfo {
        sType: STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
        queueFamilyIndex: 0,
        pNext: ptr::null(),
        flags: 0,
    };
    unsafe {
        let mut command_pool = MaybeUninit::uninit();
        ps.CreateCommandPool(device, &POOL_INFO, ptr::null(), command_pool.as_mut_ptr());
        command_pool.assume_init()
    }
}

fn create_command_buffers(
    ps: &Static,
    device: Device,
    command_pool: CommandPool,
    graphics_pipeline: Pipeline,
    render_pass: RenderPass,
    framebuffer: Framebuffer,
) -> CommandBuffer {
    const ALLOC_INFO: CommandBufferAllocateInfo = CommandBufferAllocateInfo {
        sType: STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
        commandPool: NULL_HANDLE,
        level: COMMAND_BUFFER_LEVEL_PRIMARY,
        commandBufferCount: 1,
        pNext: ptr::null(),
    };
    let mut alloc_info = ALLOC_INFO;
    alloc_info.commandPool = command_pool;
    let alloc_info = alloc_info;

    let command_buffer = unsafe {
        let mut command_buffer = MaybeUninit::uninit();
        ps.AllocateCommandBuffers(device, &alloc_info, command_buffer.as_mut_ptr());
        command_buffer.assume_init()
    };

    const BEGIN_INFO: CommandBufferBeginInfo = CommandBufferBeginInfo {
        sType: STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
        pInheritanceInfo: ptr::null(),
        pNext: ptr::null(),
        flags: 0,
    };

    unsafe {
        ps.BeginCommandBuffer(command_buffer, &BEGIN_INFO);
    }
    const RENDER_PASS_INFO: RenderPassBeginInfo = RenderPassBeginInfo {
        sType: STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
        renderPass: NULL_HANDLE,
        framebuffer: NULL_HANDLE,
        renderArea: Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent: Extent2D {
                width: 1920,
                height: 1080,
            },
        },
        pClearValues: &ClearValue {
            color: ClearColorValue {
                float32: [0f32, 0f32, 0f32, 1f32],
            },
        },
        clearValueCount: 1,
        pNext: ptr::null(),
    };
    let mut render_pass_info = RENDER_PASS_INFO;
    render_pass_info.renderPass = render_pass;
    render_pass_info.framebuffer = framebuffer;
    let render_pass_info = render_pass_info;
    unsafe {
        ps.CmdBeginRenderPass(command_buffer, &render_pass_info, SUBPASS_CONTENTS_INLINE);
        ps.CmdBindPipeline(
            command_buffer,
            PIPELINE_BIND_POINT_GRAPHICS,
            graphics_pipeline,
        );
        ps.CmdDraw(command_buffer, 3, 1, 0, 0);
        ps.CmdEndRenderPass(command_buffer);
        ps.EndCommandBuffer(command_buffer);
    }
    command_buffer
}

fn create_sync_objects(ps: &Static, device: Device) -> (Semaphore, Semaphore, Fence) {
    const SEMAPHORE_INFO: SemaphoreCreateInfo = SemaphoreCreateInfo {
        sType: STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
    };
    const FENCE_INFO: FenceCreateInfo = FenceCreateInfo {
        sType: STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
        pNext: ptr::null(),
        flags: FENCE_CREATE_SIGNALED_BIT,
    };
    unsafe {
        let mut available_semaphore = MaybeUninit::uninit();
        let mut rendered_semaphore = MaybeUninit::uninit();
        let mut in_flight_fence = MaybeUninit::uninit();
        ps.CreateSemaphore(
            device,
            &SEMAPHORE_INFO,
            ptr::null(),
            available_semaphore.as_mut_ptr(),
        );
        ps.CreateSemaphore(
            device,
            &SEMAPHORE_INFO,
            ptr::null(),
            rendered_semaphore.as_mut_ptr(),
        );
        ps.CreateFence(
            device,
            &FENCE_INFO,
            ptr::null(),
            in_flight_fence.as_mut_ptr(),
        );
        (
            available_semaphore.assume_init(),
            rendered_semaphore.assume_init(),
            in_flight_fence.assume_init(),
        )
    }
}

fn draw_frame(
    ps: &Static,
    device: Device,
    swapchain: SwapchainKHR,
    command_buffer: CommandBuffer,
    queue: Queue,
    fence: Fence,
    available_semaphore: Semaphore,
    rendered_semaphore: Semaphore,
) -> () {
    unsafe {
        ps.WaitForFences(device, 1, &fence, TRUE, u64::MAX);
    }
    let image_index = unsafe {
        let mut image_index = MaybeUninit::uninit();
        ps.AcquireNextImageKHR(
            device,
            swapchain,
            u64::MAX,
            available_semaphore,
            NULL_HANDLE,
            image_index.as_mut_ptr(),
        );
        image_index.assume_init()
    };
    const SUBMIT_INFO: SubmitInfo = SubmitInfo {
        sType: STRUCTURE_TYPE_SUBMIT_INFO,
        waitSemaphoreCount: 1,
        pWaitSemaphores: ptr::null(),
        pWaitDstStageMask: &PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
        commandBufferCount: 1,
        pCommandBuffers: ptr::null(),
        signalSemaphoreCount: 1,
        pSignalSemaphores: ptr::null(),
        pNext: ptr::null(),
    };
    let mut submit_info = SUBMIT_INFO;
    submit_info.pWaitSemaphores = &available_semaphore;
    submit_info.pSignalSemaphores = &rendered_semaphore;
    submit_info.pCommandBuffers = &command_buffer;
    let submit_info = submit_info;

    unsafe {
        ps.QueueSubmit(queue, 1, &submit_info, fence);
    }

    let mut result = MaybeUninit::uninit();
    let present_info: PresentInfoKHR = PresentInfoKHR {
        sType: STRUCTURE_TYPE_PRESENT_INFO_KHR,
        waitSemaphoreCount: 1,
        pWaitSemaphores: &rendered_semaphore,
        swapchainCount: 1,
        pSwapchains: &swapchain,
        pImageIndices: &image_index,
        pNext: ptr::null(),
        pResults: result.as_mut_ptr(),
    };
    unsafe {
        ps.QueuePresentKHR(queue, &present_info);
    }
}

#[no_mangle]
pub extern "system" fn mainCRTStartup() {
    let pointers = init();
    let (window, _hdc) = miniwin::create_window();
    init_vulkan(window, &pointers);
    loop {
        {
            if !handle_message(window) {
                break;
            }
        }

        unsafe {
            if winapi::um::winuser::GetAsyncKeyState(winapi::um::winuser::VK_ESCAPE) != 0 {
                break;
            }
        }
    }

    unsafe {
        // Tying to exit normally seems to crash after certain APIs functions have been called. ( Like ChoosePixelFormat )
        winapi::um::processthreadsapi::ExitProcess(0);
    }
}

#[no_mangle]
pub static _fltused: i32 = 1;
