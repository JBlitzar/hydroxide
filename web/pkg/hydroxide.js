/* @ts-self-types="./hydroxide.d.ts" */
import { startWorkers } from './snippets/wasm-bindgen-rayon-38edf6e439f6d70d/src/workerHelpers.js';

export class WasmRenderer {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmRendererFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmrenderer_free(ptr, 0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} size
     * @param {number} mat_type
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} fuzz
     * @param {number} refractive_index
     * @returns {number}
     */
    add_cube(x, y, z, size, mat_type, r, g, b, fuzz, refractive_index) {
        const ret = wasm.wasmrenderer_add_cube(this.__wbg_ptr, x, y, z, size, mat_type, r, g, b, fuzz, refractive_index);
        return ret >>> 0;
    }
    /**
     * @param {Uint8Array} stl_bytes
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} size
     * @param {number} mat_type
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} fuzz
     * @param {number} refractive_index
     * @returns {number}
     */
    add_mesh_stl(stl_bytes, x, y, z, size, mat_type, r, g, b, fuzz, refractive_index) {
        const ptr0 = passArray8ToWasm0(stl_bytes, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmrenderer_add_mesh_stl(this.__wbg_ptr, ptr0, len0, x, y, z, size, mat_type, r, g, b, fuzz, refractive_index);
        return ret >>> 0;
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} radius
     * @param {number} mat_type
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} fuzz
     * @param {number} refractive_index
     * @returns {number}
     */
    add_sphere(x, y, z, radius, mat_type, r, g, b, fuzz, refractive_index) {
        const ret = wasm.wasmrenderer_add_sphere(this.__wbg_ptr, x, y, z, radius, mat_type, r, g, b, fuzz, refractive_index);
        return ret >>> 0;
    }
    /**
     * @param {number} width
     * @param {number} height
     * @param {number} fov
     * @param {number} cam_x
     * @param {number} cam_y
     * @param {number} cam_z
     * @param {number} target_x
     * @param {number} target_y
     * @param {number} target_z
     * @param {number} focus_distance
     * @param {number} aperture
     * @param {number} samples
     * @param {number} termination_prob
     * @returns {Uint8Array}
     */
    export_scene(width, height, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z, focus_distance, aperture, samples, termination_prob) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmrenderer_export_scene(retptr, this.__wbg_ptr, width, height, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z, focus_distance, aperture, samples, termination_prob);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export2(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {number} index
     * @returns {Float64Array}
     */
    get_object_info(index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmrenderer_get_object_info(retptr, this.__wbg_ptr, index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayF64FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export2(r0, r1 * 8, 8);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    constructor() {
        const ret = wasm.wasmrenderer_new();
        this.__wbg_ptr = ret >>> 0;
        WasmRendererFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {number}
     */
    object_count() {
        const ret = wasm.wasmrenderer_object_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} object_index
     * @param {number} width
     * @param {number} height
     * @param {number} fov
     * @param {number} cam_x
     * @param {number} cam_y
     * @param {number} cam_z
     * @param {number} target_x
     * @param {number} target_y
     * @param {number} target_z
     * @param {number} focus_distance
     * @param {number} _aperture
     * @param {number} radius
     * @returns {Uint8Array}
     */
    outline(object_index, width, height, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z, focus_distance, _aperture, radius) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmrenderer_outline(retptr, this.__wbg_ptr, object_index, width, height, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z, focus_distance, _aperture, radius);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export2(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {number} pixel_x
     * @param {number} pixel_y
     * @param {number} width
     * @param {number} height
     * @param {number} fov
     * @param {number} cam_x
     * @param {number} cam_y
     * @param {number} cam_z
     * @param {number} target_x
     * @param {number} target_y
     * @param {number} target_z
     * @param {number} focus_distance
     * @param {number} _aperture
     * @returns {number}
     */
    pick(pixel_x, pixel_y, width, height, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z, focus_distance, _aperture) {
        const ret = wasm.wasmrenderer_pick(this.__wbg_ptr, pixel_x, pixel_y, width, height, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z, focus_distance, _aperture);
        return ret;
    }
    /**
     * @param {number} pixel_x
     * @param {number} pixel_y
     * @param {number} width
     * @param {number} height
     * @param {number} fov
     * @param {number} cam_x
     * @param {number} cam_y
     * @param {number} cam_z
     * @param {number} target_x
     * @param {number} target_y
     * @param {number} target_z
     * @returns {number}
     */
    pick_distance(pixel_x, pixel_y, width, height, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z) {
        const ret = wasm.wasmrenderer_pick_distance(this.__wbg_ptr, pixel_x, pixel_y, width, height, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z);
        return ret;
    }
    /**
     * @param {number} index
     */
    remove_object(index) {
        wasm.wasmrenderer_remove_object(this.__wbg_ptr, index);
    }
    /**
     * @param {number} width
     * @param {number} height
     * @param {number} fov
     * @param {number} cam_x
     * @param {number} cam_y
     * @param {number} cam_z
     * @param {number} target_x
     * @param {number} target_y
     * @param {number} target_z
     * @param {number} samples
     * @param {number} termination_prob
     * @param {number} focus_distance
     * @param {number} aperture
     * @returns {Uint8Array}
     */
    render(width, height, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z, samples, termination_prob, focus_distance, aperture) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmrenderer_render(retptr, this.__wbg_ptr, width, height, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z, samples, termination_prob, focus_distance, aperture);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export2(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {Uint8Array} bytes
     */
    restore(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmrenderer_restore(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {number} index
     */
    set_sky(index) {
        wasm.wasmrenderer_set_sky(this.__wbg_ptr, index);
    }
    /**
     * @param {number} hdr_index
     * @param {Uint8Array} bytes
     */
    set_sky_hdr(hdr_index, bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmrenderer_set_sky_hdr(this.__wbg_ptr, hdr_index, ptr0, len0);
    }
    /**
     * @param {Uint8Array} bytes
     */
    set_sky_hdr_bytes(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmrenderer_set_sky_hdr_bytes(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {number}
     */
    sky_count() {
        const ret = wasm.wasmrenderer_sky_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} index
     * @returns {string}
     */
    sky_name(index) {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmrenderer_sky_name(retptr, this.__wbg_ptr, index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export2(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Uint8Array}
     */
    snapshot() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmrenderer_snapshot(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export2(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {number} index
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} size
     * @param {number} mat_type
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} fuzz
     * @param {number} refractive_index
     */
    update_cube(index, x, y, z, size, mat_type, r, g, b, fuzz, refractive_index) {
        wasm.wasmrenderer_update_cube(this.__wbg_ptr, index, x, y, z, size, mat_type, r, g, b, fuzz, refractive_index);
    }
    /**
     * @param {number} index
     * @param {number} new_cx
     * @param {number} new_cy
     * @param {number} new_cz
     * @param {number} new_size
     * @param {number} rot_x
     * @param {number} rot_y
     * @param {number} rot_z
     * @param {number} mat_type
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} fuzz
     * @param {number} refractive_index
     */
    update_mesh(index, new_cx, new_cy, new_cz, new_size, rot_x, rot_y, rot_z, mat_type, r, g, b, fuzz, refractive_index) {
        wasm.wasmrenderer_update_mesh(this.__wbg_ptr, index, new_cx, new_cy, new_cz, new_size, rot_x, rot_y, rot_z, mat_type, r, g, b, fuzz, refractive_index);
    }
    /**
     * @param {number} index
     * @param {number} mat_type
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} fuzz
     * @param {number} refractive_index
     */
    update_mesh_material(index, mat_type, r, g, b, fuzz, refractive_index) {
        wasm.wasmrenderer_update_mesh_material(this.__wbg_ptr, index, mat_type, r, g, b, fuzz, refractive_index);
    }
    /**
     * @param {number} index
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} radius
     * @param {number} mat_type
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} fuzz
     * @param {number} refractive_index
     */
    update_sphere(index, x, y, z, radius, mat_type, r, g, b, fuzz, refractive_index) {
        wasm.wasmrenderer_update_sphere(this.__wbg_ptr, index, x, y, z, radius, mat_type, r, g, b, fuzz, refractive_index);
    }
}
if (Symbol.dispose) WasmRenderer.prototype[Symbol.dispose] = WasmRenderer.prototype.free;

/**
 * @param {number} num_threads
 * @returns {Promise<any>}
 */
export function initThreadPool(num_threads) {
    const ret = wasm.initThreadPool(num_threads);
    return takeObject(ret);
}

export class wbg_rayon_PoolBuilder {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(wbg_rayon_PoolBuilder.prototype);
        obj.__wbg_ptr = ptr;
        wbg_rayon_PoolBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        wbg_rayon_PoolBuilderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wbg_rayon_poolbuilder_free(ptr, 0);
    }
    build() {
        wasm.wbg_rayon_poolbuilder_build(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    numThreads() {
        const ret = wasm.wbg_rayon_poolbuilder_numThreads(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    receiver() {
        const ret = wasm.wbg_rayon_poolbuilder_receiver(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) wbg_rayon_PoolBuilder.prototype[Symbol.dispose] = wbg_rayon_PoolBuilder.prototype.free;

/**
 * @param {number} receiver
 */
export function wbg_rayon_start_worker(receiver) {
    wasm.wbg_rayon_start_worker(receiver);
}

function __wbg_get_imports(memory) {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_is_undefined_52709e72fb9f179c: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_memory_edb3f01e3930bbf6: function() {
            const ret = wasm.memory;
            return addHeapObject(ret);
        },
        __wbg___wbindgen_module_bf945c07123bafe2: function() {
            const ret = wasmModule;
            return addHeapObject(ret);
        },
        __wbg___wbindgen_throw_6ddd609b62940d55: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_instanceof_Window_23e677d2c6843922: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_startWorkers_8b582d57e92bd2d4: function(arg0, arg1, arg2) {
            const ret = startWorkers(takeObject(arg0), takeObject(arg1), wbg_rayon_PoolBuilder.__wrap(arg2));
            return addHeapObject(ret);
        },
        __wbg_static_accessor_GLOBAL_8adb955bd33fac2f: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_ad356e0db91c7913: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_SELF_f207c857566db248: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_WINDOW_bb9f1ba69d61b386: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
        memory: memory || new WebAssembly.Memory({initial:29,maximum:65536,shared:true}),
    };
    return {
        __proto__: null,
        "./hydroxide_bg.js": import0,
    };
}

const WasmRendererFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmrenderer_free(ptr >>> 0, 1));
const wbg_rayon_PoolBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wbg_rayon_poolbuilder_free(ptr >>> 0, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayF64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer !== wasm.memory.buffer) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.buffer !== wasm.memory.buffer) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.buffer !== wasm.memory.buffer) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : undefined);
if (cachedTextDecoder) cachedTextDecoder.decode();

const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().slice(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module, thread_stack_size) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    if (typeof thread_stack_size !== 'undefined' && (typeof thread_stack_size !== 'number' || thread_stack_size === 0 || thread_stack_size % 65536 !== 0)) {
        throw new Error('invalid stack size');
    }

    wasm.__wbindgen_start(thread_stack_size);
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module, memory) {
    if (wasm !== undefined) return wasm;

    let thread_stack_size
    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module, memory, thread_stack_size} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports(memory);
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module, thread_stack_size);
}

async function __wbg_init(module_or_path, memory) {
    if (wasm !== undefined) return wasm;

    let thread_stack_size
    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path, memory, thread_stack_size} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('hydroxide_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports(memory);

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module, thread_stack_size);
}

export { initSync, __wbg_init as default };
