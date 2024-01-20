struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};



struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv_cords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_cords: vec3<f32>,

    @location(1) model_matrix_0: vec4<f32>,
    @location(2) model_matrix_1: vec4<f32>,
    @location(3) model_matrix_2: vec4<f32>,
    @location(4) model_matrix_3: vec4<f32>,

};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {

  let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.uv_cords = model.position;

    out.model_matrix_0 = instance.model_matrix_0;
    out.model_matrix_1 = instance.model_matrix_1;
    out.model_matrix_2 = instance.model_matrix_2;
    out.model_matrix_3 = instance.model_matrix_3;

    return out;
}



struct Uniforms {
  cam_x : f32,
  cam_y : f32,
  cam_z : f32,
}

@group(1) @binding(1)
var voxel_data: texture_3d<u32>;
@group(1) @binding(2)
var<uniform> uniforms: Uniforms;


fn rayCubeIntersection(rayOrigin: vec3<f32>, rayDirection: vec3<f32>, cubeMin: vec3<f32>, cubeMax: vec3<f32>) -> vec3<f32> {

    if (rayOrigin.x >= cubeMin.x && rayOrigin.x <= cubeMax.x &&
        rayOrigin.y >= cubeMin.y && rayOrigin.y <= cubeMax.y &&
        rayOrigin.z >= cubeMin.z && rayOrigin.z <= cubeMax.z) {
        return rayOrigin;
    }

    var tmin: f32 = (cubeMin.x - rayOrigin.x) / rayDirection.x;
    var tmax: f32 = (cubeMax.x - rayOrigin.x) / rayDirection.x;

    if (tmin > tmax) {
      let temp = tmin;
        tmin = tmax;
        tmax = temp;
    }

    var tymin: f32 = (cubeMin.y - rayOrigin.y) / rayDirection.y;
    var tymax: f32 = (cubeMax.y - rayOrigin.y) / rayDirection.y;

    if (tymin > tymax) {
      let temp = tymin;
        tymin = tymax;
        tymax = temp;
    }

    if (tmin > tymax || tymin > tmax) {
        return vec3<f32>(0.0, 0.0, 0.0); // Intersection is nil
    }

    if (tymin > tmin) {
        tmin = tymin;
    }

    if (tymax < tmax) {
        tmax = tymax;
    }

    var tzmin: f32 = (cubeMin.z - rayOrigin.z) / rayDirection.z;
    var tzmax: f32 = (cubeMax.z - rayOrigin.z) / rayDirection.z;

    if (tzmin > tzmax) {
      let temp = tzmin;
        tzmin = tzmax;
        tzmax = temp;
    }

    if (tmin > tzmax || tzmin > tmax) {
        return vec3(100000.0); 
    }

    if (tzmin > tmin) {
        tmin = tzmin;
    }

    if (tzmax < tmax) {
        tmax = tzmax;
    }

    var intersectionPoint: vec3<f32> = rayOrigin + rayDirection * tmax;
    return intersectionPoint;
}







fn RayCast(campos: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
  let chunk_res = vec3(101.0);
  let origin = campos * chunk_res / vec3(2.0);

  let RayStepX = sqrt(1.0 + pow(dir.y / dir.x, 2.0) + pow(dir.z / dir.x, 2.0));
  let RayStepY = sqrt(1.0 + pow(dir.x / dir.y, 2.0) + pow(dir.z / dir.y, 2.0));
  let RayStepZ = sqrt(1.0 + pow(dir.x / dir.z, 2.0) + pow(dir.y / dir.z, 2.0));

  var StepVectorX : f32;
  var StepVectorY : f32;
  var StepVectorZ : f32;

  var RayLenghX : f32;
  var RayLenghY : f32;
  var RayLenghZ : f32;

  var MapCheckX = floor(origin.x);
  var MapCheckY = floor(origin.y);
  var MapCheckZ = floor(origin.z);


  if dir.x < 0.0 {
    RayLenghX = (origin.x - MapCheckX) * RayStepX;
    StepVectorX = -1.0;
  } else {
    RayLenghX = ((MapCheckX + 1.0) - origin.x) * RayStepX;
    StepVectorX = 1.0;
  }

  if dir.y < 0.0 {
    RayLenghY = (origin.y - MapCheckY) * RayStepY;
    StepVectorY = -1.0;
  } else {
    RayLenghY = ((MapCheckY + 1.0) - origin.y) * RayStepY;
    StepVectorY = 1.0;
  }

  if dir.z < 0.0 {
    RayLenghZ = (origin.z - MapCheckZ) * RayStepZ;
    StepVectorZ = -1.0;
  } else {
    RayLenghZ = ((MapCheckZ + 1.0) - origin.z) * RayStepZ;
    StepVectorZ = 1.0;
  }

  let max_dis = 1000.0;
  var current_dis = 0.0;

  while current_dis < max_dis {

    let val = textureLoad(voxel_data, vec3i(i32(MapCheckX), i32(MapCheckY), i32(MapCheckZ)), 0);

    if any(val.r != 0u) {
      return vec4(MapCheckX / chunk_res.x, MapCheckY / chunk_res.y, MapCheckZ / chunk_res.z, 1.0);
    }


    let min_ray_lengh = min(RayLenghX, min(RayLenghY, RayLenghZ));

    if min_ray_lengh == RayLenghX {
      MapCheckX += StepVectorX;
      current_dis = RayLenghX;
      RayLenghX += RayStepX;

    } else if min_ray_lengh == RayLenghY {
      MapCheckY += StepVectorY;
      current_dis = RayLenghY;
      RayLenghY += RayStepY;

    } else {
      MapCheckZ += StepVectorZ;
      current_dis = RayLenghZ;
      RayLenghZ += RayStepZ;
    }

    if MapCheckX > chunk_res.x || MapCheckY > chunk_res.y || MapCheckZ > chunk_res.y || MapCheckX < 0.0 || MapCheckY < 0.0 || MapCheckZ < 0.0 {
      break;
    }
  }

  return vec4(0.1);
}




@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let model_position = in.model_matrix_3.xyz;
  let model_rotation = mat3x3(
    in.model_matrix_0.xyz,
    in.model_matrix_1.xyz,
    in.model_matrix_2.xyz
  );


  let cam_pos = (vec3(uniforms.cam_x, uniforms.cam_y, uniforms.cam_z) - model_position) * model_rotation;
  let dir = normalize(cam_pos - in.uv_cords);
  let val = RayCast(cam_pos, dir);

  let min = vec3(-1.0);
  let max = vec3(1.0);

  let start_pos = rayCubeIntersection(cam_pos, dir, min, max) + vec3(1.0);

  let ray_res = RayCast(start_pos, dir * vec3(-1.0));

  return ray_res;
}
