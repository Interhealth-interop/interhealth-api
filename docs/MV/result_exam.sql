CREATE OR REPLACE VIEW result_exam AS
SELECT *
FROM 
 (SELECT EL.CD_PRO_FAT AS "result_exam_code",
		P.CD_PACIENTE AS "result_exam_patient_code",
		A.CD_ATENDIMENTO AS "result_exam_encounter_code",
		EL.NM_EXA_LAB AS "result_exam_description",
		NULL AS "result_exam_type",
		'Laboratório' AS "result_exam_category",
		IM.DS_ITPRE_MED AS "result_exam_condition",
		NULL AS "result_exam_status",
		NULL AS "result_exam_provider_code",
		TO_CHAR(PE.DT_EXAME, 'YYYY-MM-DD') AS "result_exam_date",
		TO_CHAR(PE.DT_EXAME, 'HH24:MI:SS') AS "result_exam_time",
		TO_CHAR(DT_LAUDO, 'YYYY-MM-DD') AS "result_exam_release_date",
		TO_CHAR(DT_LAUDO, 'HH24:MI:SS') AS "result_exam_release_time",
		PE.CD_PRE_MED AS "result_exam_document",
		NULL AS "result_exam_provider_notes",
		RE.NM_CAMPO AS "laboratory_component",
		RE.DS_RESULTADO "component_result_value",
		RE.DS_UNIDADE AS "laboratory_unit",
		RTRIM(COALESCE(RE.VL_NORMAL_ABSO_REF_INICIO || ' - ', '') || COALESCE(RE.VL_NORMAL_ABSO_REF_FIM || ' - ', '') || COALESCE(RE.DS_TEXTO_REFERENCIA_NORMAL, ''), ' -') AS "laboratory_reference",
		RE.DS_COMENTARIO AS "result_exam_notes",
		NULL AS "result_exam_request_exam_code",
		NULL AS "result_exam_document_pdf",
		NULL AS "result_exam_document_viewer",
		NULL AS "result_exam_updated_date",
		NULL AS "result_exam_reason",
		NULL AS "result_exam_esponsible"
	FROM DBAMV.PED_LAB PE
	INNER JOIN DBAMV.ATENDIME A ON A.CD_ATENDIMENTO = PE.CD_ATENDIMENTO
	INNER JOIN DBAMV.PACIENTE P ON P.CD_PACIENTE = A.CD_PACIENTE
	INNER JOIN DBAMV.RES_EXA RE ON PE.CD_PED_LAB = RE.CD_PED_LAB
	INNER JOIN DBAMV.ITPED_LAB I ON RE.CD_ITPED_LAB = I.CD_ITPED_LAB
	INNER JOIN DBAMV.EXA_LAB EL ON RE.CD_EXA_LAB = EL.CD_EXA_LAB
	LEFT JOIN (
		SELECT IM.CD_PRE_MED,
			   TP.CD_EXA_LAB,
			   IM.DS_ITPRE_MED
		FROM DBAMV.ITPRE_MED IM
		JOIN DBAMV.TIP_PRESC TP ON TP.CD_TIP_PRESC = IM.CD_TIP_PRESC
		) IM ON IM.CD_PRE_MED = PE.CD_PRE_MED
		AND IM.CD_EXA_LAB = EL.CD_EXA_LAB
	
	UNION ALL
	
	SELECT ER.EXA_RX_CD_PRO_FAT AS "result_exam_code",
		   A.CD_PACIENTE AS "result_exam_patient_code",
		   PR.CD_ATENDIMENTO AS "result_exam_encounter_code",
		   ER.DS_EXA_RX AS "result_exam_description",
		   NULL AS "result_exam_type",
		   'Imagem' AS "result_exam_category",
		   IM.DS_ITPRE_MED AS "result_exam_condition",
		   NULL AS "result_exam_status",
		   IR.CD_PRESTADOR AS "result_exam_provider_code",
		   TO_CHAR(EP.DT_CADASTRO, 'YYYY-MM-DD') AS "result_exam_date",
		   TO_CHAR(EP.DT_CADASTRO, 'HH24:MI:SS') AS "result_exam_time",
		   TO_CHAR(EP.DT_LAUDADO, 'YYYY-MM-DD') AS "result_exam_release_date",
		   TO_CHAR(EP.DT_LAUDADO, 'HH24:MI:SS') AS "result_exam_release_time",
		   PR.CD_PED_RX AS "result_exam_document",
		   EP.DS_LAUDO_TXT AS "result_exam_provider_notes",
		   NULL AS "laboratory_component",
		   NULL AS "component_result_value",
		   NULL AS "laboratory_unit",
		   NULL AS "laboratory_reference",
		   NULL AS "result_exam_notes",
		   NULL AS "result_exam_request_exam_code",
		   NULL AS "result_exam_document_pdf",
		   NULL AS "result_exam_document_viewer",
		   NULL AS "result_exam_updated_date",
		   NULL AS "result_exam_reason",
		   NULL AS "result_exam_esponsible"
	FROM IDCE.RS_LAU_EXAME_PEDIDO EP
	INNER JOIN IDCE.RS_LAU_PEDIDO_EXAME PE ON EP.ID_PEDIDO_EXAME = PE.ID_PEDIDO_EXAME
	INNER JOIN DBAMV.PED_RX PR ON PR.CD_PED_RX = PE.CD_PEDIDO_HIS
	INNER JOIN DBAMV.ITPED_RX IR ON EP.CD_ITEM_PEDIDO_HIS = IR.CD_ITPED_RX
	INNER JOIN DBAMV.ATENDIME A ON A.CD_ATENDIMENTO = PR.CD_ATENDIMENTO
	INNER JOIN DBAMV.EXA_RX ER ON ER.CD_EXA_RX = IR.CD_EXA_RX
	LEFT JOIN (
				SELECT IM.CD_PRE_MED,
					TP.CD_EXA_RX,
					IM.DS_ITPRE_MED
				FROM DBAMV.ITPRE_MED IM
				JOIN DBAMV.TIP_PRESC TP ON TP.CD_TIP_PRESC = IM.CD_TIP_PRESC
		      ) IM ON IM.CD_PRE_MED = PR.CD_PRE_MED
		AND IM.CD_EXA_RX = ER.CD_EXA_RX
		WHERE EP.DS_LAUDO_TXT IS NOT NULL
	   ) A