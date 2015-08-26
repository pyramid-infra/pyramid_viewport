peg_file! legacy_directx_x_parse("legacy_directx_x.rustpeg");
use legacy_directx_x::*;

#[test]
fn test_load_file() {
    let data = r#"xof 0303txt 0032

        Frame polySurface1 {
        	FrameTransformMatrix {
        		1.000000,0.000000,-0.000000,0.000000,0.000000,1.000000,-0.000000,0.000000,-0.000000,-0.000000,1.000000,0.000000,0.000000,0.000000,-0.000000,1.000000;;
        	}
        	Mesh polySurfaceShape1 {
        		2;
        		-4.382881;  6.532472;  1.292141;,
        		-3.229391;  6.532472;  0.391409;;
        		2;
        		3;327,326,325;,
        		4;331,330,329,328;;
        		MeshNormals {
        			2;
        			-0.028629, 0.999570, 0.006254;,
        			-0.053262, 0.998496, 0.012994;;
        			2;
        			3;127,125,126;,
        			4;124,120,121,123;;
        		}
        		MeshTextureCoords {
        			2;
        			 0.899474;-0.274396;,
        			 0.678246;-0.182760;;
        		}
        	}
        }
    "#;
    let parsed = legacy_directx_x_parse::file(data);
    let expected = DXFrame {
        name: "polySurface1".to_string(),
        transform: vec![1.000000,0.000000,-0.000000,0.000000,0.000000,1.000000,-0.000000,0.000000,-0.000000,-0.000000,1.000000,0.000000,0.000000,0.000000,-0.000000,1.000000],
        mesh: DXMesh {
            name: "polySurfaceShape1".to_string(),
            vertices: vec![
                vec![-4.382881, 6.532472, 1.292141],
                vec![-3.229391, 6.532472, 0.391409]
            ],
            indices: vec![
                vec![327,326,325],
                vec![331,330,329,328]
            ],
            normals: DXMeshNormals {
                vertices: vec![
                    vec![-0.028629, 0.999570, 0.006254],
                    vec![-0.053262, 0.998496, 0.012994]
                ],
                indices: vec![
                    vec![127,125,126],
                    vec![124,120,121,123]
                ]
            },
            texcoords: vec![
                vec![0.899474, -0.274396],
                vec![0.678246, -0.182760],
            ]
        }
    };
    assert_eq!(parsed.unwrap(), expected);
}
